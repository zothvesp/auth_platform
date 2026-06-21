-- GDPR compliance: soft delete + data retention
-- Article 17: Right to Erasure ("Right to be Forgotten")
-- Article 20: Right to Data Portability

-- ─── Soft delete on users ────────────────────────────────────────────────────
ALTER TABLE users ADD COLUMN deleted_at TIMESTAMPTZ;
CREATE INDEX users_deleted_at_idx ON users (deleted_at) WHERE deleted_at IS NOT NULL;

-- ─── Data retention config ───────────────────────────────────────────────────
INSERT INTO system_config (key, value, description, category, is_public) VALUES
  ('gdpr.retention_days',           '365',    'Days to retain soft-deleted user data before purge',  'security', false),
  ('gdpr.audit_retention_days',     '730',    'Days to retain audit logs',                          'security', false),
  ('gdpr.login_history_retention_days', '90', 'Days to retain login history',                       'security', false)
ON CONFLICT (key) DO NOTHING;
