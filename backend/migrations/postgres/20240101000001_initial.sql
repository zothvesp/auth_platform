-- AuthForge: Complete schema
-- system_config is the source of truth for all tunable settings.
-- Secrets (keys, passwords) stay in env. Everything else can live here.

-- ─── Extensions ──────────────────────────────────────────────────────────────
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm"; -- for user search

-- ─── System Config ────────────────────────────────────────────────────────────
-- Stores every tunable setting. env vars are the fallback, not the primary.
-- Frontend fetches /api/v1/config/public for validation rules & feature flags.
CREATE TABLE system_config (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    description TEXT NOT NULL,
    category    TEXT NOT NULL,          -- auth | security | email | features | validation
    is_public   BOOLEAN NOT NULL DEFAULT false, -- safe to expose to unauthenticated clients
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default config values (env vars override at startup via upsert)
INSERT INTO system_config (key, value, description, category, is_public) VALUES
  -- Auth
  ('auth.jwt_access_expiry_secs',   '900',    'JWT access token lifetime in seconds',        'auth',       false),
  ('auth.jwt_refresh_expiry_secs',  '604800', 'JWT refresh token lifetime in seconds',        'auth',       false),
  ('auth.max_login_attempts',       '5',      'Failed attempts before account lock',          'security',   false),
  ('auth.lockout_duration_secs',    '900',    'Account lockout duration in seconds',          'security',   false),
  ('auth.require_email_verification','true',  'Block login until email is verified',          'auth',       true),
  ('auth.allow_registration',       'true',   'Allow new user self-registration',             'auth',       true),
  -- Password policy (public — shown to users in the UI)
  ('password.min_length',           '8',      'Minimum password length',                      'validation', true),
  ('password.require_uppercase',    'true',   'Require at least one uppercase letter',        'validation', true),
  ('password.require_lowercase',    'true',   'Require at least one lowercase letter',        'validation', true),
  ('password.require_number',       'true',   'Require at least one number',                  'validation', true),
  ('password.require_special',      'true',   'Require at least one special character',       'validation', true),
  -- Field length limits (public)
  ('validation.display_name_min',   '2',      'Minimum display name length',                  'validation', true),
  ('validation.display_name_max',   '50',     'Maximum display name length',                  'validation', true),
  ('validation.role_name_min',      '2',      'Minimum role name length',                     'validation', true),
  ('validation.role_name_max',      '50',     'Maximum role name length',                     'validation', true),
  -- OAuth feature flags (public)
  ('oauth.google_enabled',          'false',  'Enable Google OAuth',                          'features',   true),
  ('oauth.github_enabled',          'false',  'Enable GitHub OAuth',                          'features',   true),
  ('oauth.microsoft_enabled',       'false',  'Enable Microsoft OAuth',                       'features',   true),
  ('oauth.saml_enabled',            'false',  'Enable SAML 2.0',                              'features',   true),
  -- MFA
  ('mfa.enabled',                   'true',   'Allow users to set up MFA',                    'features',   true),
  ('mfa.enforce_for_admins',        'false',  'Require MFA for admin roles',                  'security',   true),
  -- Email
  ('email.verification_expiry_hrs', '24',     'Email verification token expiry in hours',     'email',      false),
  ('email.reset_expiry_mins',       '15',     'Password reset token expiry in minutes',       'email',      false),
  -- Session
  ('session.cookie_secure',         'false',  'Require Secure flag on cookies (prod)',        'auth',       false),
  ('session.same_site',             'Strict', 'SameSite cookie policy',                       'auth',       false)
ON CONFLICT (key) DO NOTHING;

-- ─── Users ────────────────────────────────────────────────────────────────────
CREATE TABLE users (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email            TEXT NOT NULL,
    display_name     TEXT NOT NULL,
    password_hash    TEXT,
    avatar_url       TEXT,
    email_verified   BOOLEAN NOT NULL DEFAULT false,
    status           TEXT NOT NULL DEFAULT 'active'
                     CHECK (status IN ('active','inactive','suspended')),
    mfa_enabled      BOOLEAN NOT NULL DEFAULT false,
    mfa_secret       TEXT,
    auth_method      TEXT NOT NULL DEFAULT 'password'
                     CHECK (auth_method IN ('password','google','github','microsoft','saml','oidc')),
    last_login_at    TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX users_email_idx ON users (LOWER(email));
CREATE INDEX users_status_idx ON users (status);

-- ─── Permissions ──────────────────────────────────────────────────────────────
CREATE TABLE permissions (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource    TEXT NOT NULL,
    action      TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (resource, action)
);
CREATE INDEX permissions_resource_idx ON permissions (resource);

-- ─── Roles ───────────────────────────────────────────────────────────────────
CREATE TABLE roles (
    id             UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name           TEXT NOT NULL UNIQUE,
    description    TEXT NOT NULL,
    is_system      BOOLEAN NOT NULL DEFAULT false,
    parent_role_id UUID REFERENCES roles(id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX roles_name_idx ON roles (name);

-- ─── Role ↔ Permission (many-to-many) ────────────────────────────────────────
CREATE TABLE role_permissions (
    role_id       UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- ─── User ↔ Role (many-to-many) ──────────────────────────────────────────────
CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

-- ─── Refresh Tokens ───────────────────────────────────────────────────────────
CREATE TABLE refresh_tokens (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    family      UUID NOT NULL,
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX refresh_tokens_user_idx    ON refresh_tokens (user_id);
CREATE INDEX refresh_tokens_family_idx  ON refresh_tokens (family);
CREATE INDEX refresh_tokens_expires_idx ON refresh_tokens (expires_at);

-- ─── Email Verification Tokens ────────────────────────────────────────────────
CREATE TABLE email_verification_tokens (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX email_tokens_user_idx ON email_verification_tokens (user_id);

-- ─── Password Reset Tokens ────────────────────────────────────────────────────
CREATE TABLE password_reset_tokens (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX reset_tokens_user_idx ON password_reset_tokens (user_id);

-- ─── OAuth Accounts ───────────────────────────────────────────────────────────
CREATE TABLE oauth_accounts (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider         TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    provider_email   TEXT,
    access_token     TEXT,
    refresh_token    TEXT,
    token_expires_at TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (provider, provider_user_id)
);
CREATE INDEX oauth_accounts_user_idx ON oauth_accounts (user_id);

-- ─── Sessions ─────────────────────────────────────────────────────────────────
CREATE TABLE sessions (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ip_address TEXT NOT NULL,
    user_agent TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX sessions_user_idx    ON sessions (user_id);
CREATE INDEX sessions_expires_idx ON sessions (expires_at);

-- ─── Audit Logs ───────────────────────────────────────────────────────────────
CREATE TABLE audit_logs (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID REFERENCES users(id) ON DELETE SET NULL,
    user_email  TEXT,
    action      TEXT NOT NULL,
    resource    TEXT NOT NULL,
    resource_id TEXT,
    ip_address  TEXT NOT NULL,
    user_agent  TEXT NOT NULL,
    success     BOOLEAN NOT NULL,
    details     JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX audit_logs_user_idx     ON audit_logs (user_id);
CREATE INDEX audit_logs_action_idx   ON audit_logs (action);
CREATE INDEX audit_logs_created_idx  ON audit_logs (created_at DESC);

-- ─── Login History ────────────────────────────────────────────────────────────
CREATE TABLE login_history (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ip_address  TEXT NOT NULL,
    user_agent  TEXT NOT NULL,
    location    TEXT,
    success     BOOLEAN NOT NULL,
    auth_method TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX login_history_user_idx    ON login_history (user_id);
CREATE INDEX login_history_created_idx ON login_history (created_at DESC);

-- ─── MFA Backup Codes ─────────────────────────────────────────────────────────
CREATE TABLE backup_codes (
    id        UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash TEXT NOT NULL,
    used_at   TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX backup_codes_user_idx ON backup_codes (user_id);
