# AuthForge — Authentication & Identity Platform

Production-ready auth platform: **React + TanStack** frontend, **Rust + Axum** backend.

---

## Quick Start

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.75+ | `apt install rustc cargo` |
| Node.js | 20+ | https://nodejs.org |
| Docker | 24+ | https://docs.docker.com/get-docker |
| PostgreSQL | 16 | via Docker below |

---

### 1. Start Infrastructure

```bash
# From project root
docker-compose up -d          # starts postgres:5432 + redis:6379
# Optional pgAdmin at http://localhost:5050
docker-compose --profile tools up -d
```

---

### 2. Backend Setup

```bash
cd backend
cp .env.example .env          # fill in your values (RSA keys pre-generated in .env)

# Generate fresh RSA keys (optional — .env already has dev keys)
openssl genrsa -out private.pem 2048
openssl rsa -in private.pem -pubout -out public.pem
# Paste into .env as single-line with \n escapes

# Install sqlx-cli for migrations (first time only)
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Run migrations
sqlx migrate run --source migrations/postgres

# Start the server
cargo run
# → http://localhost:8080
# → Health check: http://localhost:8080/health
```

---

### 3. Frontend Setup

```bash
cd frontend
npm install
npm run dev
# → http://localhost:5173
```

---

## Architecture

```
auth-platform/
├── backend/                    # Rust / Axum
│   ├── src/
│   │   ├── config.rs           # env → typed config (secrets only)
│   │   ├── db.rs               # PgPool + RedisPool
│   │   ├── error.rs            # AppError → HTTP responses
│   │   ├── middleware/
│   │   │   └── auth.rs         # JWT Bearer extractor + permission guards
│   │   ├── models/             # DB row types (one file per aggregate)
│   │   ├── repositories/       # ALL sqlx queries live here only
│   │   ├── routes/             # Thin HTTP handlers → call services
│   │   └── services/           # Business logic (no SQL)
│   ├── migrations/postgres/    # PostgreSQL schema + seeds
│   └── .env.example
│
└── frontend/                   # React / TypeScript
    └── src/
        ├── components/
        │   ├── auth/           # OAuthButtons, PasswordStrength
        │   ├── layout/         # DashboardLayout (sidebar + nav)
        │   ├── rbac/           # PermissionGate
        │   └── ui/             # Button, FormField, Toaster, etc.
        ├── hooks/
        │   └── useConfig.ts    # usePublicConfig, useFeatureFlags, usePasswordPolicy
        ├── lib/
        │   ├── api.ts          # Axios client + all API modules
        │   ├── config.ts       # Dynamic Zod schema builders (from server config)
        │   └── queryClient.ts  # TanStack Query client + key factories
        ├── routes/             # TanStack Router file-based routes
        │   ├── __root.tsx
        │   ├── _auth.tsx       # Protected layout (requires login)
        │   ├── _auth/          # Dashboard pages
        │   ├── _public.tsx     # Public layout (redirects if logged in)
        │   └── _public/        # Login, register, forgot-password, etc.
        ├── store/
        │   └── authStore.ts    # Auth state + useAuth hook
        └── utils/
            └── permissions.ts  # P.* permission constants, usePermission hooks
```

---

## Design Principles

### Repository Pattern
- **Only `src/repositories/` contains `sqlx::query*` calls** — nowhere else
- Services receive `&PgPool` from `AppState`, pass it to repositories
- Every repository implements `BaseRepository<Model>` (find_by_id, find_all, delete, get, exists)

### DB-First Config
- All tunables live in the `system_config` table (password policy, feature flags, expiry times)
- Secrets (JWT keys, SMTP password) stay in env vars — never in DB
- Frontend fetches `GET /api/v1/config/public` on boot; Zod validation schemas are built dynamically from the response
- Services access config via `services::config::*` — never `state.config.*` for tunables

### Permission Flow
- Server computes the full flattened permission set after role hierarchy expansion
- JWT contains `permissions: ["users:read", ...]` — no client-side hierarchy logic
- `PermissionGate` and `usePermission` check `user.permissions[]` directly
- No hardcoded role bypass in frontend (`super_admin` logic is server-side only)

---

## API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | — | Health check |
| POST | `/api/v1/auth/register` | — | Register new user |
| POST | `/api/v1/auth/login` | — | Login with email/password |
| POST | `/api/v1/auth/logout` | Bearer | Logout + revoke refresh token |
| POST | `/api/v1/auth/refresh` | Cookie | Rotate refresh token |
| GET | `/api/v1/auth/verify/:token` | — | Verify email address |
| POST | `/api/v1/auth/forgot-password` | — | Send password reset email |
| POST | `/api/v1/auth/reset-password` | — | Reset password with token |
| POST | `/api/v1/auth/check-permission` | Bearer | Check single permission |
| POST | `/api/v1/auth/check-permissions` | Bearer | Batch permission check |
| GET | `/api/v1/config/public` | — | Password policy + feature flags |
| GET | `/api/v1/config` | Bearer+settings:manage | All system config |
| PUT | `/api/v1/config/:key` | Bearer+settings:manage | Update config value |
| GET | `/api/v1/users/me` | Bearer | Get current user |
| PUT | `/api/v1/users/me` | Bearer | Update profile |
| POST | `/api/v1/users/me/change-password` | Bearer | Change password |
| GET | `/api/v1/roles` | Bearer+roles:read | List all roles |
| POST | `/api/v1/roles` | Bearer+roles:create | Create role |
| GET/PUT/DELETE | `/api/v1/roles/:id` | Bearer+roles:* | Manage role |
| GET | `/api/v1/permissions` | Bearer+permissions:read | List permissions |
| POST | `/api/v1/permissions` | Bearer+permissions:manage | Create permission |
| GET | `/api/v1/admin/users` | Bearer+users:read | Paginated user list |
| PATCH | `/api/v1/admin/users/:id/deactivate` | Bearer+users:manage | Deactivate user |
| POST | `/api/v1/admin/users/:id/roles` | Bearer+users:manage | Assign role to user |
| GET | `/api/v1/admin/audit-logs` | Bearer+audit:read | Paginated audit logs |

---

## Default Roles

| Role | Inherits | Permissions |
|------|----------|-------------|
| `super_admin` | — | All (bypassed server-side) |
| `admin` | — | users:manage, roles:manage, permissions:manage, audit:read, settings:manage, oauth_apps:manage |
| `manager` | — | users:read, users:update, audit:read |
| `user` | — | (none — standard authenticated user) |
| `viewer` | — | users:read, roles:read, permissions:read |

---

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | ✅ | PostgreSQL connection string |
| `REDIS_URL` | ✅ | Redis connection string |
| `JWT_PRIVATE_KEY` | ✅ | RSA-2048 private key (PKCS#8 PEM, `\n` escaped) |
| `JWT_PUBLIC_KEY` | ✅ | RSA-2048 public key (PEM, `\n` escaped) |
| `COOKIE_DOMAIN` | ✅ | Cookie domain (e.g. `localhost`) |
| `ALLOWED_ORIGINS` | ✅ | Comma-separated CORS origins |
| `APP_BASE_URL` | ✅ | Frontend URL (used in emails) |
| `SMTP_HOST` | — | SMTP server host |
| `GOOGLE_CLIENT_ID` | — | Enable with `oauth.google_enabled=true` in system_config |

See `backend/.env.example` for the full list.

---

## TanStack Libraries Used

| Library | Purpose |
|---------|---------|
| `@tanstack/react-router` | File-based routing, `beforeLoad` auth guards, search params |
| `@tanstack/react-query` | Server state, caching, background refresh, mutations |
| `@tanstack/react-form` | Form state, field-level validation with Zod, async submit |
| `@tanstack/react-table` | Sortable/filterable tables for users, roles, audit logs |
| `@tanstack/react-virtual` | Virtualized lists for large datasets |
| `@tanstack/router-devtools` | Route inspection in development |
| `@tanstack/react-query-devtools` | Cache inspection in development |

---

## Development Commands

```bash
# Backend
cargo check                    # type-check without building
cargo run                      # start with hot-reload via cargo-watch
cargo test                     # run unit tests

# Frontend
npm run dev                    # Vite dev server with HMR
npm run build                  # Production build
npm run typecheck              # tsc --noEmit

# Database
sqlx migrate run               # apply migrations
sqlx migrate revert            # rollback last migration
sqlx migrate add <name>        # create new migration
```

---

*AuthForge — built with Rust 1.75+ · React 19 · TanStack Router/Query/Form/Table*
