# AuthForge

Authentication & Identity Platform — self-hosted auth server with JWT, OAuth2/OIDC, SAML, MFA, RBAC, and GDPR compliance.

## Stack

| Layer | Technology |
|---|---|
| Backend | Rust 1.75+ / Axum 0.7 / SQLx 0.7 / PostgreSQL 16 / Redis 7 |
| Frontend | Next.js 15 (App Router) + Refine + Mantine 5 + TanStack Table |
| Infra | Docker Compose (Postgres `:5433`, Redis `:6378`, Vault `:8200`) |

## Features

- **JWT Auth** — RS256 access tokens, refresh token rotation with family-based reuse detection
- **OAuth2/OIDC** — Google, GitHub, Microsoft providers with PKCE + CSRF
- **SAML 2.0** — Service provider with AuthnRequest/Response parsing
- **MFA** — TOTP with backup codes, admin-enforceable
- **RBAC** — Hierarchical roles with recursive CTE permission expansion
- **Vault Integration** — HashiCorp Vault (KV v2 + Transit) or local AES-256-GCM fallback
- **GDPR** — Soft delete, data export (Article 20), account erasure with PII anonymization (Article 17), configurable retention
- **Rate Limiting** — Per-path Redis-backed limits (login: 5/15min, register: 5/15min)
- **Audit Logging** — All security and admin events recorded
- **DB-First Config** — Runtime tunables in `system_config` table, secrets in Vault/env

## Quick Start

```bash
# First time setup
make setup          # generates keys + vault master key, installs deps

# Start infrastructure
make db-up          # Postgres + Redis + Vault via Docker Compose

# Run migrations
cd backend && sqlx migrate run --source migrations/postgres

# Seed database
make seed           # creates roles, permissions, demo users, OAuth apps

# Run dev servers
make dev            # starts backend (cargo-watch) + frontend (next dev)
```

## Port Mappings

Docker Compose remaps host ports: Postgres `5433→5432`, Redis `6378→6379`, Vault `8200→8200`.

## Demo Credentials

| Email | Password | Role |
|---|---|---|
| superadmin@authforge.dev | Admin@1234! | super_admin |
| admin@authforge.dev | Admin@1234! | admin |
| manager@authforge.dev | Admin@1234! | manager |
| user@authforge.dev | Admin@1234! | user |
| viewer@authforge.dev | Admin@1234! | viewer |

## Architecture

```
HTTP Request
     │
     ▼
┌─────────────────────────────────────────────┐
│  Routes  (thin controllers — no SQL/logic)  │
└──────────────────┬──────────────────────────┘
                   │ calls
                   ▼
┌─────────────────────────────────────────────┐
│  Services  (business logic, orchestration)  │
└──────────────────┬──────────────────────────┘
                   │ calls
                   ▼
┌─────────────────────────────────────────────┐
│  Repositories  (only place sqlx queries live)│
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│  Vault  (JWT keys, secret encryption)       │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│  Database  (Postgres + Redis)               │
└─────────────────────────────────────────────┘
```

## Vault Setup

Set `VAULT_ADDR=http://localhost:8200` and `VAULT_TOKEN=authforge-dev-token` in `.env` to use HashiCorp Vault. Without these, falls back to local AES-256-GCM encryption with `VAULT_MASTER_KEY`.

```bash
# Initialize Vault engines (after docker-compose up)
docker compose exec vault sh /vault-init.sh
```

## API Endpoints

### Auth
- `POST /api/v1/auth/register` — Create account
- `POST /api/v1/auth/login` — Sign in (supports MFA)
- `POST /api/v1/auth/logout` — Sign out
- `POST /api/v1/auth/refresh` — Refresh access token
- `POST /api/v1/auth/forgot-password` — Request reset email
- `POST /api/v1/auth/reset-password` — Reset with token

### User Self-Service
- `GET /api/v1/users/me` — Current user profile
- `PUT /api/v1/users/me` — Update profile
- `POST /api/v1/users/me/change-password` — Change password
- `GET /api/v1/users/me/export` — Export all data (GDPR)
- `DELETE /api/v1/users/me/delete` — Delete account (GDPR)

### Admin
- `GET /api/v1/admin/users` — List users (paginated)
- `POST /api/v1/admin/users` — Create user
- `GET /api/v1/admin/roles` — List roles
- `GET /api/v1/admin/audit-logs` — View audit logs

### OAuth2/OIDC Provider
- `GET /.well-known/openid-configuration` — OIDC discovery
- `GET /oauth/jwks` — JSON Web Key Set
- `POST /oauth/token` — Token endpoint
- `GET /oauth/userinfo` — UserInfo endpoint
- `POST /oauth/introspect` — Token introspection (RFC 7662)
- `POST /oauth/revoke` — Token revocation (RFC 7009)

## License

Proprietary — internal use only.
