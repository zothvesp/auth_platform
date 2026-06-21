-- JWT signing keys table for automated key rotation
CREATE TABLE IF NOT EXISTS jwt_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kid TEXT NOT NULL UNIQUE,
    public_key_pem TEXT NOT NULL,
    private_key_pem TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at TIMESTAMPTZ
);

-- Index for finding current active key
CREATE INDEX IF NOT EXISTS idx_jwt_keys_active ON jwt_keys (created_at DESC) WHERE rotated_at IS NULL;
