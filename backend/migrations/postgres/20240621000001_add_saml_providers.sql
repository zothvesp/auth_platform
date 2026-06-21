-- SAML 2.0 providers table
CREATE TABLE saml_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    entity_id TEXT NOT NULL,
    sso_url TEXT NOT NULL,
    certificate TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX saml_providers_name_idx ON saml_providers (name);
