-- OAuth2/OIDC Provider tables

-- Registered OAuth applications (clients)
CREATE TABLE oauth_apps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id VARCHAR(64) UNIQUE NOT NULL,
    client_secret_hash VARCHAR(128) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    redirect_uris TEXT[] NOT NULL,
    allowed_grants TEXT[] NOT NULL DEFAULT '{authorization_code}',
    allowed_scopes TEXT[] NOT NULL DEFAULT '{openid,profile,email}',
    pkce_required BOOLEAN NOT NULL DEFAULT true,
    logo_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX oauth_apps_client_id_idx ON oauth_apps (client_id);

-- Short-lived authorization codes
CREATE TABLE authorization_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code_hash VARCHAR(128) UNIQUE NOT NULL,
    client_id VARCHAR(64) NOT NULL REFERENCES oauth_apps(client_id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    redirect_uri TEXT NOT NULL,
    scope TEXT NOT NULL,
    code_challenge TEXT,
    code_challenge_method VARCHAR(10),
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX authorization_codes_hash_idx ON authorization_codes (code_hash);
CREATE INDEX authorization_codes_expires_idx ON authorization_codes (expires_at);
