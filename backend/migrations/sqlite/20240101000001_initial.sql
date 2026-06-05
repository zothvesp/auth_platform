-- AuthForge SQLite schema
-- Used for local development and CI. Production uses PostgreSQL.
-- Differences from PostgreSQL:
--   • No uuid_generate_v4() — UUIDs generated in application layer
--   • BOOLEAN stored as INTEGER (0/1)
--   • TIMESTAMPTZ stored as TEXT (ISO 8601)
--   • JSONB stored as TEXT
--   • No pg_trgm extension

-- ─── System Config ─────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS system_config (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category    TEXT NOT NULL DEFAULT 'custom',
    is_public   INTEGER NOT NULL DEFAULT 0,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO system_config (key, value, description, category, is_public) VALUES
  ('auth.jwt_access_expiry_secs',    '900',    'JWT access token lifetime',         'auth',       0),
  ('auth.jwt_refresh_expiry_secs',   '604800', 'JWT refresh token lifetime',         'auth',       0),
  ('auth.max_login_attempts',        '5',      'Failed attempts before lockout',     'security',   0),
  ('auth.lockout_duration_secs',     '900',    'Account lockout duration',           'security',   0),
  ('auth.require_email_verification','true',   'Block login until email verified',   'auth',       1),
  ('auth.allow_registration',        'true',   'Allow new user registration',        'auth',       1),
  ('password.min_length',            '8',      'Minimum password length',            'validation', 1),
  ('password.require_uppercase',     'true',   'Require uppercase letter',           'validation', 1),
  ('password.require_lowercase',     'true',   'Require lowercase letter',           'validation', 1),
  ('password.require_number',        'true',   'Require number',                     'validation', 1),
  ('password.require_special',       'true',   'Require special character',          'validation', 1),
  ('validation.display_name_min',    '2',      'Min display name length',            'validation', 1),
  ('validation.display_name_max',    '50',     'Max display name length',            'validation', 1),
  ('validation.role_name_min',       '2',      'Min role name length',               'validation', 1),
  ('validation.role_name_max',       '50',     'Max role name length',               'validation', 1),
  ('oauth.google_enabled',           'false',  'Enable Google OAuth',                'features',   1),
  ('oauth.github_enabled',           'false',  'Enable GitHub OAuth',                'features',   1),
  ('oauth.microsoft_enabled',        'false',  'Enable Microsoft OAuth',             'features',   1),
  ('oauth.saml_enabled',             'false',  'Enable SAML 2.0',                   'features',   1),
  ('mfa.enabled',                    'true',   'Allow users to set up MFA',          'features',   1),
  ('mfa.enforce_for_admins',         'false',  'Require MFA for admins',             'security',   1),
  ('email.verification_expiry_hrs',  '24',     'Email verification expiry hours',    'email',      0),
  ('email.reset_expiry_mins',        '15',     'Password reset expiry minutes',      'email',      0),
  ('session.cookie_secure',          'false',  'Require Secure cookie flag',         'auth',       0),
  ('session.same_site',              'Strict', 'SameSite cookie policy',             'auth',       0);

-- ─── Users ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id             TEXT PRIMARY KEY,
    email          TEXT NOT NULL,
    display_name   TEXT NOT NULL,
    password_hash  TEXT,
    avatar_url     TEXT,
    email_verified INTEGER NOT NULL DEFAULT 0,
    status         TEXT NOT NULL DEFAULT 'active'
                   CHECK (status IN ('active','inactive','suspended')),
    mfa_enabled    INTEGER NOT NULL DEFAULT 0,
    mfa_secret     TEXT,
    auth_method    TEXT NOT NULL DEFAULT 'password'
                   CHECK (auth_method IN ('password','google','github','microsoft','saml','oidc')),
    last_login_at  TEXT,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS users_email_idx ON users (email COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS users_status_idx ON users (status);

-- ─── Permissions ───────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS permissions (
    id          TEXT PRIMARY KEY,
    resource    TEXT NOT NULL,
    action      TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (resource, action)
);

-- ─── Roles ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS roles (
    id             TEXT PRIMARY KEY,
    name           TEXT NOT NULL UNIQUE,
    description    TEXT NOT NULL DEFAULT '',
    is_system      INTEGER NOT NULL DEFAULT 0,
    parent_role_id TEXT REFERENCES roles(id) ON DELETE SET NULL,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Role ↔ Permission ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id       TEXT NOT NULL REFERENCES roles(id)       ON DELETE CASCADE,
    permission_id TEXT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- ─── User ↔ Role ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

-- ─── Refresh Tokens ─────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    family     TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    used_at    TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS refresh_tokens_user_idx   ON refresh_tokens (user_id);
CREATE INDEX IF NOT EXISTS refresh_tokens_family_idx ON refresh_tokens (family);

-- ─── Email Verification Tokens ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS email_verification_tokens (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE UNIQUE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ─── Password Reset Tokens ──────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    used_at    TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS reset_tokens_user_idx ON password_reset_tokens (user_id);

-- ─── OAuth Accounts ─────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS oauth_accounts (
    id               TEXT PRIMARY KEY,
    user_id          TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider         TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    provider_email   TEXT,
    access_token     TEXT,
    refresh_token    TEXT,
    token_expires_at TEXT,
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (provider, provider_user_id)
);
CREATE INDEX IF NOT EXISTS oauth_accounts_user_idx ON oauth_accounts (user_id);

-- ─── Sessions ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS sessions (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ip_address TEXT NOT NULL,
    user_agent TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS sessions_user_idx ON sessions (user_id);

-- ─── Audit Logs ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS audit_logs (
    id          TEXT PRIMARY KEY,
    user_id     TEXT REFERENCES users(id) ON DELETE SET NULL,
    user_email  TEXT,
    action      TEXT NOT NULL,
    resource    TEXT NOT NULL,
    resource_id TEXT,
    ip_address  TEXT NOT NULL,
    user_agent  TEXT NOT NULL,
    success     INTEGER NOT NULL,
    details     TEXT,  -- JSON stored as TEXT in SQLite
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS audit_logs_user_idx    ON audit_logs (user_id);
CREATE INDEX IF NOT EXISTS audit_logs_created_idx ON audit_logs (created_at);

-- ─── Login History ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS login_history (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ip_address  TEXT NOT NULL,
    user_agent  TEXT NOT NULL,
    location    TEXT,
    success     INTEGER NOT NULL,
    auth_method TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS login_history_user_idx ON login_history (user_id);

-- ─── MFA Backup Codes ───────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS backup_codes (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash  TEXT NOT NULL,
    used_at    TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS backup_codes_user_idx ON backup_codes (user_id);
