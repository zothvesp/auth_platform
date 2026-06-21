//! SAML 2.0 service — AuthnRequest/Response handling.
//! No SQL here. All DB access goes through repositories.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use quick_xml::events::Event;
use quick_xml::Reader;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::pkcs8::DecodePublicKey;
use rsa::sha2::Sha256;
use rsa::signature::Verifier;
use rsa::RsaPublicKey;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::SamlProvider,
    repositories::{
        base::BaseRepository, OAuthRepository, RoleRepository, SamlRepository, UserRepository,
    },
    services,
    state::AppState,
};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct SamlLoginRequest {
    pub url: String,
    pub relay_state: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SamlCallbackParams {
    pub saml_response: String,
    pub relay_state: Option<String>,
}

// ─── Initiate SAML Login ──────────────────────────────────────────────────────

pub async fn initiate_saml_login(
    state: &AppState,
    provider_name: &str,
) -> AppResult<SamlLoginRequest> {
    let provider = SamlRepository::new(&state.db.pool)
        .find_by_name(provider_name)
        .await?
        .ok_or_else(|| AppError::NotFound("SAML provider".to_string()))?;

    if !provider.enabled {
        return Err(AppError::Forbidden);
    }

    let relay_state = Uuid::new_v4().to_string();
    let request_id = Uuid::new_v4().to_string();

    let authn_request = build_authn_request(&request_id, &provider.entity_id, &provider.sso_url);

    let encoded = BASE64.encode(authn_request.as_bytes());

    let redirect_url = format!(
        "{}?SAMLRequest={}&RelayState={}",
        provider.sso_url,
        urlencoding::encode(&encoded),
        urlencoding::encode(&relay_state),
    );

    Ok(SamlLoginRequest {
        url: redirect_url,
        relay_state,
    })
}

// ─── Handle SAML Callback ─────────────────────────────────────────────────────

pub async fn handle_saml_callback(
    state: &AppState,
    saml_response: &str,
    ip: &str,
    user_agent: &str,
) -> AppResult<(services::auth::UserDto, String, String)> {
    let decoded = BASE64
        .decode(saml_response)
        .map_err(|_| AppError::InvalidToken)?;
    let xml = String::from_utf8(decoded).map_err(|_| AppError::InvalidToken)?;

    let claims = parse_saml_response(&xml)?;

    // Verify XML signature if provider has a certificate configured
    if let Some(ref issuer) = claims.provider {
        if let Some(provider) = SamlRepository::new(&state.db.pool)
            .find_by_entity_id(issuer)
            .await?
        {
            if !provider.certificate.is_empty() {
                let has_signature =
                    xml.contains("<Signature") || xml.contains("<ds:Signature");
                if !has_signature {
                    return Err(AppError::OAuth(
                        "SAML response missing required signature".to_string(),
                    ));
                }
                verify_xml_element_signature(&xml, &provider.certificate)?;
            }
        }
    }

    let email = claims
        .email
        .ok_or_else(|| AppError::OAuth("SAML response missing email".to_string()))?;

    let display_name = claims
        .name
        .unwrap_or_else(|| email.clone());

    let provider_name = claims
        .provider
        .unwrap_or_else(|| "saml".to_string());

    // Upsert identity (same pattern as OAuth)
    let user_id = upsert_saml_identity(state, &provider_name, &claims.subject, &email, &display_name).await?;

    let user = UserRepository::new(&state.db.pool).get(user_id).await?;
    if user.status != "active" {
        return Err(AppError::AccountInactive);
    }

    UserRepository::update_last_login(&state.db.pool, user_id).await?;
    UserRepository::record_login(
        &state.db.pool,
        user_id,
        ip,
        user_agent,
        true,
        &provider_name,
    )
    .await?;

    services::auth::issue_session(state, user_id, ip, user_agent).await
}

// ─── XML Building ─────────────────────────────────────────────────────────────

fn build_authn_request(request_id: &str, issuer: &str, acs_url: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
                    xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
                    ID="{}"
                    Version="2.0"
                    IssueInstant="{}"
                    Destination="{}"
                    AssertionConsumerServiceURL="{}"
                    ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
  <saml:Issuer>{}</saml:Issuer>
  <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"
                      AllowCreate="true"/>
</samlp:AuthnRequest>"#,
        request_id,
        Utc::now().to_rfc3339(),
        acs_url,
        acs_url,
        issuer,
    )
}

// ─── XML Signature Verification ──────────────────────────────────────────────

fn verify_xml_element_signature(xml: &str, certificate_pem: &str) -> AppResult<()> {
    let signed_info = extract_xml_element(xml, "SignedInfo").ok_or_else(|| {
        AppError::OAuth("Missing SignedInfo in SAML signature".to_string())
    })?;

    let sig_value_element =
        extract_xml_element(xml, "SignatureValue").ok_or_else(|| {
            AppError::OAuth("Missing SignatureValue in SAML signature".to_string())
        })?;
    let sig_value_b64 = extract_text_content(&sig_value_element);
    let sig_bytes = BASE64
        .decode(sig_value_b64.trim())
        .map_err(|_| AppError::OAuth("Invalid SignatureValue encoding".to_string()))?;

    let public_key = parse_x509_public_key(certificate_pem)?;

    let canonical = canonicalize_xml(&signed_info);

    let vk = VerifyingKey::<Sha256>::new(public_key);
    let sig = Signature::try_from(sig_bytes.as_slice())
        .map_err(|_| AppError::OAuth("Invalid RSA signature format".to_string()))?;
    vk.verify(canonical.as_bytes(), &sig)
        .map_err(|_| {
            AppError::OAuth("SAML XML signature verification failed".to_string())
        })?;

    Ok(())
}

fn extract_xml_element(xml: &str, tag_name: &str) -> Option<String> {
    for prefix in &["", "ds:", "dsig:", "sig:"] {
        let open_prefix = format!("<{}{}", prefix, tag_name);
        if let Some(start) = xml.find(&open_prefix) {
            let after_prefix = start + open_prefix.len();
            if after_prefix >= xml.len() {
                continue;
            }
            let next_byte = xml.as_bytes()[after_prefix];
            if next_byte.is_ascii_alphanumeric() {
                continue;
            }
            let open_end = xml[after_prefix..].find('>')? + after_prefix;
            let content_start = open_end + 1;
            let close_tag = format!("</{}{}>", prefix, tag_name);
            let close_pos = xml[content_start..].find(&close_tag)? + content_start;
            let close_end = close_pos + close_tag.len();
            return Some(xml[start..close_end].to_string());
        }
    }
    None
}

fn extract_text_content(element: &str) -> &str {
    if let Some(start) = element.find('>') {
        if let Some(end) = element.rfind("</") {
            return &element[start + 1..end];
        }
    }
    ""
}

fn canonicalize_xml(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find("<!--") {
        result.push_str(&rest[..pos]);
        if let Some(end) = rest[pos + 4..].find("-->") {
            rest = &rest[pos + 4 + end + 3..];
        } else {
            rest = &rest[pos + 4..];
        }
    }
    result.push_str(rest);
    result
        .replace("\r\n", "\n")
        .replace("\r", "\n")
        .trim()
        .to_string()
}

fn parse_x509_public_key(pem: &str) -> AppResult<RsaPublicKey> {
    let cleaned = pem.trim();
    let b64: String = cleaned
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");

    let der = BASE64
        .decode(&b64)
        .map_err(|_| AppError::OAuth("Invalid certificate PEM encoding".to_string()))?;

    let spki = extract_spki_from_x509(&der).map_err(|_| {
        AppError::OAuth("Failed to extract public key from X.509 certificate".to_string())
    })?;

    RsaPublicKey::from_public_key_der(&spki)
        .map_err(|e| AppError::OAuth(format!("Invalid RSA public key: {}", e)))
}

fn der_read_length(data: &[u8], offset: usize) -> Result<(usize, usize), ()> {
    if offset >= data.len() {
        return Err(());
    }
    let first = data[offset];
    if first & 0x80 == 0 {
        Ok((1, first as usize))
    } else {
        let num_bytes = (first & 0x7F) as usize;
        if num_bytes == 0 || offset + 1 + num_bytes > data.len() {
            return Err(());
        }
        let mut length = 0usize;
        for i in 0..num_bytes {
            length = (length << 8) | data[offset + 1 + i] as usize;
        }
        Ok((1 + num_bytes, length))
    }
}

fn der_read_header(data: &[u8], offset: usize) -> Result<(u8, usize, usize, usize), ()> {
    if offset >= data.len() {
        return Err(());
    }
    let tag = data[offset];
    let (len_bytes, length) = der_read_length(data, offset + 1)?;
    let header_len = 1 + len_bytes;
    Ok((tag, header_len, offset + header_len, length))
}

fn der_skip_element(data: &[u8], offset: usize) -> Result<usize, ()> {
    let (_, _header_len, value_offset, value_len) = der_read_header(data, offset)?;
    Ok(value_offset + value_len)
}

fn extract_spki_from_x509(cert_der: &[u8]) -> Result<Vec<u8>, ()> {
    let (_, _, seq_offset, seq_len) = der_read_header(cert_der, 0)?;
    if cert_der[0] & 0x20 == 0 {
        return Err(());
    }

    let mut pos = seq_offset;
    let end = seq_offset + seq_len;

    // Skip version [0] EXPLICIT if present
    if pos < end && cert_der[pos] == 0xA0 {
        pos = der_skip_element(cert_der, pos)?;
    }

    // Skip serialNumber, signature, issuer, validity, subject
    for _ in 0..5 {
        pos = der_skip_element(cert_der, pos)?;
    }

    let (_, _header_len, spki_offset, spki_len) = der_read_header(cert_der, pos)?;
    Ok(cert_der[pos..spki_offset + spki_len].to_vec())
}

// ─── XML Parsing ──────────────────────────────────────────────────────────────

struct SamlClaims {
    subject: String,
    email: Option<String>,
    name: Option<String>,
    provider: Option<String>,
}

fn parse_saml_response(xml: &str) -> AppResult<SamlClaims> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut claims = SamlClaims {
        subject: String::new(),
        email: None,
        name: None,
        provider: None,
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag.as_str() {
                    "NameID" => {
                        if let Ok(text) = reader.read_text(e.name()) {
                            claims.subject = text.to_string();
                        }
                    }
                    "Attribute" => {
                        let name_attr = e.attributes().flatten().find_map(|a| {
                            if a.key.as_ref() == b"Name" {
                                Some(String::from_utf8_lossy(&a.value).to_string())
                            } else {
                                None
                            }
                        });
                        if let Some(attr_name) = name_attr {
                            if let Ok(text) = reader.read_text(e.name()) {
                                match attr_name.as_str() {
                                    "email" | "emailAddress" => {
                                        claims.email = Some(text.to_string());
                                    }
                                    "displayName" | "name" => {
                                        claims.name = Some(text.to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    "Issuer" => {
                        if let Ok(text) = reader.read_text(e.name()) {
                            claims.provider = Some(text.to_string());
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if claims.subject.is_empty() {
        return Err(AppError::OAuth(
            "SAML response missing subject".to_string(),
        ));
    }

    Ok(claims)
}

// ─── Identity Management ──────────────────────────────────────────────────────

async fn upsert_saml_identity(
    state: &AppState,
    provider_name: &str,
    subject: &str,
    email: &str,
    display_name: &str,
) -> AppResult<Uuid> {
    // Check if already linked
    let oauth_repo = OAuthRepository::new(&state.db.pool);
    if let Some(account) = oauth_repo
        .find_by_provider(provider_name, subject)
        .await?
    {
        return Ok(account.user_id);
    }

    let user_repo = UserRepository::new(&state.db.pool);
    let user_id = if let Some(user) = user_repo.find_by_email(email).await? {
        user.id
    } else {
        let user_id = Uuid::new_v4();
        let mut tx = state.db.pool.begin().await?;
        UserRepository::create(
            &mut *tx,
            user_id,
            email,
            display_name,
            None,
            provider_name,
        )
        .await?;
        UserRepository::set_email_verified(&mut *tx, user_id).await?;
        if let Some(role) = RoleRepository::new(&state.db.pool)
            .find_by_name("user")
            .await?
        {
            UserRepository::assign_role(&mut *tx, user_id, role.id, None).await?;
        }
        tx.commit().await?;
        user_id
    };

    OAuthRepository::upsert(
        &state.db.pool,
        user_id,
        provider_name,
        subject,
        Some(email),
        None,
        None,
        None,
    )
    .await?;

    Ok(user_id)
}

// ─── Provider Management ──────────────────────────────────────────────────────

pub async fn list_providers(state: &AppState) -> AppResult<Vec<SamlProvider>> {
    SamlRepository::new(&state.db.pool).find_all().await
}

pub async fn create_provider(
    state: &AppState,
    name: &str,
    display_name: &str,
    entity_id: &str,
    sso_url: &str,
    certificate: &str,
) -> AppResult<SamlProvider> {
    SamlRepository::new(&state.db.pool)
        .create(name, display_name, entity_id, sso_url, certificate)
        .await
}

pub async fn toggle_provider(state: &AppState, id: Uuid, enabled: bool) -> AppResult<()> {
    SamlRepository::new(&state.db.pool)
        .update_enabled(id, enabled)
        .await
}

pub async fn delete_provider(state: &AppState, id: Uuid) -> AppResult<()> {
    SamlRepository::new(&state.db.pool).delete(id).await
}

// ─── Metadata Generation ─────────────────────────────────────────────────────

pub async fn generate_metadata(state: &AppState, provider_name: &str) -> AppResult<String> {
    let provider = SamlRepository::new(&state.db.pool)
        .find_by_name(provider_name)
        .await?
        .ok_or_else(|| AppError::NotFound("SAML provider".to_string()))?;

    let entity_id = &provider.entity_id;
    let acs_url = format!(
        "{}/api/v1/auth/saml/callback",
        state.config.app_base_url
    );

    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata"
                     entityID="{entity_id}">
  <md:SPSSODescriptor
      AuthnRequestsSigned="false"
      WantAssertionsSigned="true"
      protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat>
    <md:AssertionConsumerService
        Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
        Location="{acs_url}"
        index="0"
        isDefault="true"/>
  </md:SPSSODescriptor>
</md:EntityDescriptor>"#,
        entity_id = entity_id,
        acs_url = acs_url,
    ))
}
