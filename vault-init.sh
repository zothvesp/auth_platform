#!/bin/sh
# Vault initialization script for AuthForge
# Run after Vault is healthy: docker compose exec vault sh /vault-init.sh

set -e

export VAULT_ADDR="http://localhost:8200"
export VAULT_TOKEN="authforge-dev-token"

echo "==> Enabling KV v2 secrets engine at secret/..."
vault secrets enable -path=secret kv-v2 2>/dev/null || echo "KV v2 already enabled"

echo "==> Enabling Transit engine at transit/..."
vault secrets enable -path=transit transit 2>/dev/null || echo "Transit already enabled"

echo "==> Creating transit encryption key 'authforge-master'..."
vault write -f transit/keys/authforge-master type=aes256-gcm 2>/dev/null || echo "Key already exists"

echo "==> Storing initial JWT keys placeholder..."
vault kv put secret/authforge/jwt-keys \
  current_kid="initial" \
  public_key_pem="" \
  private_key_pem="" 2>/dev/null || echo "JWT keys already stored"

echo "==> Storing OAuth client secrets..."
vault kv put secret/authforge/oauth-secrets \
  google_client_secret="" \
  github_client_secret="" \
  microsoft_client_secret="" 2>/dev/null || echo "OAuth secrets already stored"

echo "==> Storing SMTP credentials..."
vault kv put secret/authforge/smtp \
  smtp_password="" 2>/dev/null || echo "SMTP secrets already stored"

echo "==> Vault initialization complete!"
echo ""
echo "Dev root token: authforge-dev-token"
echo "Vault UI: http://localhost:8200/ui"
echo ""
echo "Usage:"
echo "  vault kv get secret/authforge/jwt-keys"
echo "  vault kv put secret/authforge/oauth-secrets google_client_secret=xxx"
echo "  vault write -f transit/encrypt/authforge-master plaintext=$(echo -n 'test' | base64)"
