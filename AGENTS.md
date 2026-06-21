# AGENTS.md — AuthForge

## Stack

- **Backend**: Rust 1.75+ / Axum / SQLx / PostgreSQL 16 / Redis 7
- **Frontend**: Next.js 15 (App Router) + Refine + Mantine 5 + TanStack Table
- **Infra**: Docker Compose (Postgres on `:5433`, Redis on `:6378`)

## Quick Commands

```bash
# Full setup (first time)
make setup          # keys + npm install + .env copy

# Start infra
make db-up          # Postgres + Redis via Docker Compose

# Migrations
cd backend && sqlx migrate run --source migrations/postgres

# Run dev servers
make dev            # kills existing servers, starts both (cargo-watch + pnpm dev)
make stop           # kills running dev servers

# Seed
make seed           # runs: cargo run --bin seed

# Verify
make lint           # clippy (backend) + next lint (frontend)
make test           # cargo test (backend) + pnpm typecheck (frontend)
```

## Port Mappings (non-obvious)

Docker Compose **remaps** host ports: Postgres `5433→5432`, Redis `6378→6379`. Your `.env` `DATABASE_URL` must use `localhost:5432` (container-internal), not `:5433`.

## Backend Rules

**Architecture is enforced by `backend/CLAUDE.md`** — read it before changing backend code.

Key constraints:
- `sqlx::query*` calls exist **only** in `src/repositories/`. If you see them in services/routes, that's a bug.
- Routes are thin controllers: parse → call service → return. No SQL, no business logic.
- Services orchestrate repositories, own business logic. No `sqlx` imports.
- Repositories receive `&PgPool`, not `&AppState`.
- DTOs live alongside the service that produces them (`src/services/<name>.rs`), not in `models/`.
- Config tunables: use `services::config::*`, never `state.config.*` for runtime settings.

Two binaries exist: `authforge` (main server) and `seed` (DB seeder).

## Frontend Rules

- Frontend is a **Next.js App Router** project using **Refine** (`@refinedev/*`), not raw TanStack Router.
- Providers wiring: `src/app/providers.tsx` → Refine + Mantine + Auth Provider + Data Provider.
- Auth provider: `src/providers/auth-provider/`. Data provider: `src/providers/data-provider/`.
- Path aliases: `@*` → `./src/*`, `@pages/*` → `./pages/*` (defined in `tsconfig.json`).
- The `predev` script auto-deletes `.next/` before dev — don't do it manually.
- ESLint: extends `next/core-web-vitals` (`.eslintrc.json`).
- Frontend uses pnpm (not npm) for all commands in the Makefile, though README says `npm run dev`.

## DB-First Config Pattern

Tunables (password policy, feature flags, expiry times) live in the `system_config` table.
Secrets (JWT keys, SMTP, OAuth secrets) stay in `.env`. Frontend fetches public config at boot via `GET /api/v1/config/public` and builds Zod validation schemas dynamically from the response.

## Permission System

- Server computes flattened permission set after role hierarchy expansion.
- JWT contains `permissions: ["users:read", ...]`. Frontend reads `user.permissions[]` directly.
- No client-side role hierarchy logic. No hardcoded `super_admin` bypass in frontend.
- Permission constants: backend `models/config.rs` keys module, frontend `utils/permissions.ts` (`P.*` / `R.*`). Must stay in sync with seed migrations.

## Migration Files

- PostgreSQL migrations: `backend/migrations/postgres/`
- SQLite migrations: `backend/migrations/sqlite/` (exists but may be unused)
- New migration: `sqlx migrate add <name>`, then edit the generated file.

## Gotchas

- `npm run dev` in README is misleading — the Makefile uses `pnpm dev`. Use `pnpm`.
- Backend `.env` has pre-generated RSA dev keys — no need to regenerate for local dev.
- `docker-compose.yml` uses `version: "3.9"` (legacy key) and `docker compose` (v2 CLI) interchangeably in the Makefile.
- The `CHOKIDAR_USEPOLLING=true` env var on `dev-frontend` is needed for file watching in certain environments (Docker, WSL).
