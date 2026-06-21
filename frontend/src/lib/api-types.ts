// Shared TypeScript types matching backend wire formats.
// Single source of truth for frontend API types.
//
// DTOs with #[serde(rename_all = "camelCase")] → camelCase field names.
// Models without rename_all → snake_case field names.
//
// Backend DTOs: backend/src/services/{auth,admin,oauth_apps,oauth,oauth_provider,saml}.rs
// Backend models: backend/src/models/

// ─── Auth DTOs (services/auth.rs — camelCase) ───────────────────────────────

export interface UserDto {
  id: string;
  email: string;
  displayName: string;
  avatarUrl: string | null;
  emailVerified: boolean;
  status: string;
  mfaEnabled: boolean;
  authMethod: string;
  lastLoginAt: string | null;
  createdAt: string;
  roles: RoleDto[];
  permissions: string[];
}

export interface RoleDto {
  id: string;
  name: string;
  description: string;
}

export interface Claims {
  sub: string;
  email: string;
  roles: string[];
  permissions: string[];
  iat: number;
  exp: number;
  jti: string;
}

// ─── Admin DTOs (services/admin.rs — camelCase) ─────────────────────────────

export interface CreateUserInput {
  email: string;
  displayName: string;
  password: string;
  emailVerified?: boolean;
  status?: string;
  roleIds?: string[];
}

export interface PaginatedUsers {
  data: UserDto[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export interface PaginatedAuditLogs {
  data: AuditLog[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export interface BulkUserResult {
  affected: number;
}

// ─── OAuth App DTOs (services/oauth_apps.rs — camelCase) ────────────────────

export interface CreateOAuthAppInput {
  name: string;
  description?: string;
  redirectUris: string[];
  allowedGrants: string[];
  allowedScopes: string[];
  pkceRequired?: boolean;
  logoUrl?: string;
}

export interface UpdateOAuthAppInput {
  name?: string;
  description?: string | null;
  redirectUris?: string[];
  allowedGrants?: string[];
  allowedScopes?: string[];
  pkceRequired?: boolean;
  logoUrl?: string | null;
  isActive?: boolean;
}

export interface OAuthAppResponse {
  id: string;
  clientId: string;
  name: string;
  description: string | null;
  redirectUris: string[];
  allowedGrants: string[];
  allowedScopes: string[];
  pkceRequired: boolean;
  logoUrl: string | null;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateOAuthAppResponse extends OAuthAppResponse {
  clientSecret: string;
}

// ─── OAuth Flow DTOs (services/oauth.rs — camelCase) ────────────────────────

export interface AuthorizationResponse {
  url: string;
}

export interface LinkedAccount {
  provider: string;
  providerEmail: string | null;
  createdAt: string;
}

// ─── OAuth Provider DTOs (services/oauth_provider.rs — camelCase) ────────────

export interface AuthorizationEndpointResponse {
  redirectTo: string;
}

export interface AuthorizationRequest {
  responseType: string;
  clientId: string;
  redirectUri: string;
  scope?: string;
  state?: string;
  codeChallenge?: string;
  codeChallengeMethod?: string;
}

export interface TokenRequest {
  grantType: string;
  code?: string;
  redirectUri?: string;
  clientId: string;
  clientSecret?: string;
  codeVerifier?: string;
}

export interface TokenResponse {
  accessToken: string;
  tokenType: string;
  expiresIn: number;
  refreshToken?: string;
  scope: string;
  idToken?: string;
}

export interface UserInfoResponse {
  sub: string;
  email?: string;
  name?: string;
  emailVerified?: boolean;
}

export interface OidcConfiguration {
  issuer: string;
  authorizationEndpoint: string;
  tokenEndpoint: string;
  userinfoEndpoint: string;
  jwksUri: string;
  scopesSupported: string[];
  responseTypesSupported: string[];
  grantTypesSupported: string[];
  subjectTypesSupported: string[];
  idTokenSigningAlgValuesSupported: string[];
  tokenEndpointAuthMethodsSupported: string[];
  claimsSupported: string[];
}

export interface JwksResponse {
  keys: JwksKey[];
}

export interface JwksKey {
  kty: string;
  alg: string;
  use: string;
  kid: string;
  n: string;
  e: string;
}

export interface IntrospectionResponse {
  active: boolean;
  sub?: string;
  clientId?: string;
  scope?: string;
  exp?: number;
  iat?: number;
  iss?: string;
}

// ─── SAML DTOs (services/saml.rs — camelCase) ───────────────────────────────

export interface SamlLoginRequest {
  url: string;
  relayState: string;
}

export interface SamlCallbackParams {
  samlResponse: string;
  relayState?: string;
}

// ─── Models (serialized directly — snake_case) ──────────────────────────────

export interface AuditLog {
  id: string;
  user_id: string | null;
  user_email: string | null;
  action: string;
  resource: string;
  resource_id: string | null;
  ip_address: string;
  user_agent: string;
  success: boolean;
  details: Record<string, unknown> | null;
  created_at: string;
}

export interface Permission {
  id: string;
  resource: string;
  action: string;
  description: string;
  created_at: string;
}

export interface Role {
  id: string;
  name: string;
  description: string;
  is_system: boolean;
  parent_role_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface RoleResponse extends Role {
  permissions: Permission[];
}

export interface SamlProvider {
  id: string;
  name: string;
  display_name: string;
  entity_id: string;
  sso_url: string;
  certificate: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface OAuthApp {
  id: string;
  client_id: string;
  name: string;
  description: string | null;
  redirect_uris: string[];
  allowed_grants: string[];
  allowed_scopes: string[];
  pkce_required: boolean;
  logo_url: string | null;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface SystemConfig {
  key: string;
  value: string;
  description: string;
  category: string;
  is_public: boolean;
  updated_at: string;
}

export interface Session {
  id: string;
  user_id: string;
  ip_address: string;
  user_agent: string;
  expires_at: string;
  created_at: string;
}

// ─── Public Config (models/config.rs — snake_case) ──────────────────────────

export interface PublicConfig {
  password_policy: PasswordPolicy;
  validation_rules: ValidationRules;
  features: FeatureFlags;
}

export interface PasswordPolicy {
  min_length: number;
  require_uppercase: boolean;
  require_lowercase: boolean;
  require_number: boolean;
  require_special: boolean;
}

export interface ValidationRules {
  display_name_min: number;
  display_name_max: number;
  role_name_min: number;
  role_name_max: number;
}

export interface FeatureFlags {
  allow_registration: boolean;
  require_email_verification: boolean;
  oauth_google: boolean;
  oauth_github: boolean;
  oauth_microsoft: boolean;
  saml_enabled: boolean;
  mfa_enabled: boolean;
  mfa_enforce_for_admins: boolean;
}
