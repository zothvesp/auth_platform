# AuthForge — Authentication & Identity Platform
## Production Build Plan

> **Stack**: React (TanStack ecosystem) + Rust (Axum) | Full-stack auth & RBAC system

---

## Current Implementation Snapshot

Status legend used below:

| Status | Meaning |
|--------|---------|
| ✅ Done | Implemented and present in the codebase |
| 🟨 Partial | Scaffolded or partly functional, but missing important behavior, tests, UI, or production hardening |
| ⬜ Todo | Not implemented yet |

### Completed Core

- Backend scaffold, Axum routing, SQLx migrations, PostgreSQL schema, seed command, RSA JWT access tokens, refresh-token rotation, logout, email verification, password reset, RBAC tables, role/permission APIs, user profile/admin APIs, and public config endpoints are present.
- Frontend scaffold, Next.js App Router + Refine + Mantine 5, TanStack Query client, auth store, fetch API client with refresh handling, protected routes, permission gate, login/register/forgot/reset/verify pages, dashboard shell, profile/security pages, and React 19 local dependency setup are present.
- Local dev infra exists for PostgreSQL and Redis via Docker Compose.
- OAuth2 client: Google, GitHub, Microsoft login fully implemented with PKCE, profile fetch, identity upsert.
- SAML 2.0: AuthnRequest + response parsing implemented (no XML signature verification).
- MFA: Real TOTP via `totp-rs`, backup codes, enable/disable/verify flows.
- OIDC Provider: `/.well-known/openid-configuration`, `/oauth/authorize`, `/oauth/token`, `/oauth/userinfo` endpoints.
- Frontend: Skeleton loading states, i18n with ~130 translatable strings, account linking UI, admin OAuth config UI, shared request helper.

### Known Gaps

- OAuth callback page exists for social login; OIDC consent page not yet implemented.
- JWKS endpoint returns empty keys (placeholder).
- SAML has no XML signature verification.
- No CSP headers, CSRF protection, or dependency audit in CI.
- `backend/src/services/auth.rs` is too broad — should split JWT, password, email, rate-limiting into separate modules.
- Audit logging table exists but mutations don't consistently write audit events.
- Rate limiting only covers login — no per-IP or per-route middleware.
- Docker Compose runs infra only, not backend/frontend app containers.
- No backend tests (only 2 MFA unit tests), no frontend tests, no E2E tests.
- No OpenAPI spec or API documentation.

### Backend Engineering Audit

Backend is mostly aligned with GRASP/KISS:

- Routes are thin.
- Services hold business logic.
- Repositories own SQL.
- Models are separate.
- Config is centralized.

Main backend issues to track:

- `backend/src/services/auth.rs` is getting too broad: password hashing, JWT, refresh tokens, email verification, password reset, rate limiting, and OIDC token issuance are all accumulating there. Should be split into focused modules.
- `backend/src/routes/admin.rs` uses typed user update DTOs; keep new admin mutations typed.
- OAuth client, SAML, and MFA are implemented — keep them isolated as they mature.
- OIDC Provider JWKS needs real RSA key parsing.
- Audit logging needs to be wired into mutation paths.

### Next Task

**Wire audit logging into mutations** — the `audit_logs` table and `AuditRepository::create` exist, but no service method writes events during role/user mutations. After that, focus on CSP headers, CSRF protection, and backend tests.

## Project Overview

AuthForge is a production-ready authentication and identity management platform with role-based access control. It supports JWT, Sessions, OAuth 2.0, SSO, and SAML 2.0 authentication standards.

---

## Phase 1 — Project Setup & Infrastructure

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 1.1 | Monorepo Setup | Init workspace with pnpm/turborepo; `apps/frontend`, `apps/backend`, `packages/shared-types` | 🔴 Critical | M | ⬜ Todo |
| 1.2 | Frontend Scaffold | Vite + React 18 + TypeScript; configure path aliases | 🔴 Critical | S | ✅ Done |
| 1.3 | Backend Scaffold | Cargo workspace; Axum + Tokio + SQLx; configure feature flags | 🔴 Critical | M | ✅ Done |
| 1.4 | Database Setup | PostgreSQL schema; migrations via SQLx; seed data | 🔴 Critical | L | ✅ Done |
| 1.5 | Docker Compose | `postgres`, `redis`, `backend`, `frontend` services for local dev | 🟡 High | M | 🟨 Partial |
| 1.6 | CI/CD Pipeline | GitHub Actions: lint, test, build, deploy on merge | 🟡 High | L | ⬜ Todo |
| 1.7 | Env Config | `.env` management; secrets vault integration points | 🟡 High | S | 🟨 Partial |
| 1.8 | Shared Types Package | Shared TypeScript + Rust (via `typeshare`) type definitions | 🟢 Medium | M | ⬜ Todo |

---

## Phase 2 — Core Authentication Backend (Rust/Axum)

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 2.1 | User Model & DB | `users` table: id, email, password_hash, status, verified_at, created_at | 🔴 Critical | M | ✅ Done |
| 2.2 | Password Hashing | Argon2id via `argon2` crate; configurable work factor | 🔴 Critical | S | 🟨 Partial |
| 2.3 | Registration Endpoint | `POST /auth/register`; validation, uniqueness check, email verification token | 🔴 Critical | M | ✅ Done |
| 2.4 | Login Endpoint | `POST /auth/login`; credential validation, rate limiting, account lock | 🔴 Critical | M | ✅ Done |
| 2.5 | JWT Implementation | Access token (15min) + refresh token (7d); RS256 signing; `jsonwebtoken` crate | 🔴 Critical | L | ✅ Done |
| 2.6 | JWT Middleware | Axum extractor; validate signature, expiry, claims; attach user context | 🔴 Critical | M | ✅ Done |
| 2.7 | Session-Based Auth | Redis-backed sessions; `POST /auth/session`; cookie with HttpOnly + Secure | 🟡 High | L | ⬜ Todo |
| 2.8 | Token Refresh | `POST /auth/refresh`; rotate refresh tokens; revoke old tokens | 🔴 Critical | M | ✅ Done |
| 2.9 | Logout | `POST /auth/logout`; invalidate tokens/sessions; clear cookies | 🟡 High | S | ✅ Done |
| 2.10 | Email Verification | `GET /auth/verify/:token`; time-limited token; re-send endpoint | 🟡 High | M | ✅ Done |
| 2.11 | Password Reset | `POST /auth/forgot-password` + `POST /auth/reset-password`; HMAC token | 🟡 High | M | 🟨 Partial |
| 2.12 | Rate Limiting | `tower-governor` or custom middleware; per-IP, per-user limits | 🔴 Critical | M | 🟨 Partial |

---

## Phase 3 — OAuth 2.0 & SSO Backend

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 3.1 | OAuth2 Framework | Generic OAuth2 client abstraction; state param; PKCE support | 🔴 Critical | L | ✅ Done |
| 3.2 | Google OAuth | `GET /auth/oauth/google` + callback; user upsert | 🟡 High | M | ✅ Done |
| 3.3 | GitHub OAuth | `GET /auth/oauth/github` + callback | 🟡 High | M | ✅ Done |
| 3.4 | Microsoft/Azure AD | `GET /auth/oauth/microsoft` + callback; tenant config | 🟡 High | L | ✅ Done |
| 3.5 | OIDC/SSO Provider | OpenID Connect discovery; `/.well-known/openid-configuration` | 🟡 High | L | 🟨 Partial (JWKS placeholder) |
| 3.6 | SAML 2.0 SP | Service Provider impl; assertion parsing | 🟢 Medium | XL | 🟨 Partial (no XML signature verification) |
| 3.7 | SAML IdP Metadata | `GET /auth/saml/metadata`; XML generation | 🟢 Medium | M | ⬜ Todo |
| 3.8 | OAuth App Management | CRUD for OAuth apps; client_id/secret generation | 🟢 Medium | L | 🟨 Partial (DB + repo exist, no admin UI) |
| 3.9 | Token Introspection | `POST /oauth/introspect`; RFC 7662 compliance | 🟢 Medium | M | ⬜ Todo |
| 3.10 | Token Revocation | `POST /oauth/revoke`; RFC 7009 compliance | 🟢 Medium | S | ⬜ Todo |

---

## Phase 4 — RBAC & Permissions Backend

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 4.1 | Roles Table | `roles`: id, name, description, parent_role_id, is_system | 🔴 Critical | M | ✅ Done |
| 4.2 | Permissions Table | `permissions`: id, resource, action, description | 🔴 Critical | M | ✅ Done |
| 4.3 | Role-Permission Join | `role_permissions` many-to-many; inheritance via parent_role | 🔴 Critical | M | ✅ Done |
| 4.4 | User-Role Join | `user_roles` with optional scope/context | 🔴 Critical | S | 🟨 Partial |
| 4.5 | Permission Resolver | Recursive role hierarchy traversal; permission set computation | 🔴 Critical | L | ✅ Done |
| 4.6 | RBAC Middleware | Axum guard extractor; `require_permission!` macro | 🔴 Critical | M | 🟨 Partial |
| 4.7 | Roles API | `GET/POST /roles`, `GET/PUT/DELETE /roles/:id` | 🟡 High | M | ✅ Done |
| 4.8 | Permissions API | `GET/POST /permissions`, `GET/PUT/DELETE /permissions/:id` | 🟡 High | M | 🟨 Partial |
| 4.9 | Role Assignment API | `POST/DELETE /users/:id/roles` | 🟡 High | S | 🟨 Partial |
| 4.10 | Permission Check API | `POST /auth/check-permission`; batch permission checks | 🟡 High | M | ✅ Done |
| 4.11 | Default Roles Seed | Seed: `super_admin`, `admin`, `manager`, `user`, `viewer` | 🟡 High | S | ✅ Done |
| 4.12 | Audit Logging | `audit_logs` table; log all permission/role changes | 🟢 Medium | L | 🟨 Partial |

---

## Phase 5 — Identity Management Backend

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 5.1 | User Profile API | `GET/PUT /users/me`; avatar, display name, preferences | 🔴 Critical | M | 🟨 Partial |
| 5.2 | User CRUD (Admin) | `GET/PUT/DELETE /admin/users/:id`; paginated list | 🟡 High | M | 🟨 Partial |
| 5.3 | User Search | `GET /admin/users?q=&role=&status=`; full-text search | 🟡 High | M | 🟨 Partial |
| 5.4 | Account Deactivation | Soft delete; `PATCH /admin/users/:id/deactivate` | 🟡 High | S | ✅ Done |
| 5.5 | Account Deletion | GDPR-compliant hard delete with data scrub | 🟢 Medium | M | 🟨 Partial |
| 5.6 | MFA Setup | TOTP via `totp-rs`; `POST /auth/mfa/setup`, `/verify`, `/disable` | 🟢 Medium | XL | ✅ Done |
| 5.7 | Backup Codes | Generate + validate one-time backup codes for MFA recovery | 🟢 Medium | M | ⬜ Todo |
| 5.8 | Activity Log | Login history, IP, device info per user | 🟢 Medium | M | 🟨 Partial |

---

## Phase 6 — Frontend Architecture (React + Next.js)

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 6.1 | Next.js App Router Setup | File-based routing via App Router; route groups; lazy loading | 🔴 Critical | M | ✅ Done |
| 6.2 | TanStack Query Setup | QueryClient config; devtools; error boundaries | 🔴 Critical | S | 🟨 Partial |
| 6.3 | Auth Store | Cookie-based session; auth provider via Refine | 🔴 Critical | M | ✅ Done |
| 6.4 | API Client | Fetch wrapper with auto token attach; 401 interceptor | 🔴 Critical | L | ✅ Done |
| 6.5 | Protected Route | Refine `Authenticated` wrapper; redirect to `/login` | 🔴 Critical | M | ✅ Done |
| 6.6 | Permission Guard | `<PermissionGate>` HOC + `usePermission` hook | 🔴 Critical | M | ✅ Done |
| 6.7 | Forms | Refine + Mantine form integration | 🟡 High | L | ✅ Done |
| 6.8 | Tables | Refine + TanStack Table via `DataTable` component | 🟡 High | L | ✅ Done |
| 6.9 | Error Boundary | Global + route-level error boundaries with fallback UI | 🟡 High | M | ⬜ Todo |
| 6.10 | Toast/Notification | Mantine notifications for auth events | 🟡 High | S | ✅ Done |
| 6.11 | Skeleton Loading | Skeleton placeholders replacing spinners on all pages | 🟡 High | M | ✅ Done |
| 6.12 | i18n | Translations file + `useTranslations` hook; ~130 strings | 🟡 High | L | ✅ Done |

---

## Phase 7 — Frontend Auth Pages

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 7.1 | Login Page | Email/password + social login buttons; remember me | 🔴 Critical | M | 🟨 Partial |
| 7.2 | Register Page | Full registration form with TanStack Form + validation | 🔴 Critical | M | ✅ Done |
| 7.3 | Forgot Password | Email submission page | 🔴 Critical | S | ✅ Done |
| 7.4 | Reset Password | Token-verified new password form | 🔴 Critical | S | 🟨 Partial |
| 7.5 | Email Verification | Verify page + resend option | 🟡 High | S | 🟨 Partial |
| 7.6 | OAuth Callback | Handle provider redirects; exchange code; store tokens | 🔴 Critical | M | ✅ Done |
| 7.7 | SSO Login | Enterprise SSO entry; domain-based provider discovery | 🟡 High | M | ⬜ Todo |
| 7.8 | MFA Prompt | TOTP code entry page; backup code fallback | 🟢 Medium | M | ⬜ Todo |
| 7.9 | Session Expired | Auto-redirect with session expiry notification | 🟡 High | S | 🟨 Partial |
| 7.10 | Unauthorized (403) | Friendly 403 page with role/permission context | 🟡 High | S | 🟨 Partial |

---

## Phase 8 — Frontend Dashboard & Admin UI

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 8.1 | Dashboard Layout | Sidebar nav; breadcrumbs; header with user menu | 🔴 Critical | L | 🟨 Partial |
| 8.2 | Profile Page | Edit profile; change password; connected accounts | 🟡 High | M | 🟨 Partial |
| 8.3 | Users Admin Page | TanStack Table with search, filter, sort; bulk actions | 🟡 High | L | 🟨 Partial |
| 8.4 | User Detail Page | View/edit user; assign roles; activity log | 🟡 High | M | 🟨 Partial |
| 8.5 | Roles Admin Page | Create/edit/delete roles; permission matrix | 🟡 High | L | 🟨 Partial |
| 8.6 | Permissions Admin | View all permissions; grouped by resource | 🟡 High | M | ⬜ Todo |
| 8.7 | OAuth Apps Page | Register/manage OAuth applications | 🟢 Medium | M | ⬜ Todo |
| 8.8 | Audit Log Page | TanStack Table for audit events; date range filter | 🟢 Medium | M | ⬜ Todo |
| 8.9 | Security Settings | MFA setup; active sessions; trusted devices | 🟢 Medium | L | 🟨 Partial |
| 8.10 | System Settings | SMTP config; OAuth provider config; SAML metadata | 🟢 Medium | XL | ⬜ Todo |

---

## Phase 9 — Security & Hardening

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 9.1 | CORS Configuration | Strict origin allowlist; preflight handling | 🔴 Critical | S | ✅ Done |
| 9.2 | CSP Headers | Content Security Policy; nonce-based script | 🔴 Critical | M | ⬜ Todo |
| 9.3 | CSRF Protection | Double-submit cookie pattern for session auth | 🔴 Critical | M | ⬜ Todo |
| 9.4 | Input Sanitization | Server-side: `validator` crate; XSS prevention | 🔴 Critical | M | 🟨 Partial |
| 9.5 | SQL Injection | Parameterized queries via SQLx (enforced by type system) | 🔴 Critical | S | ✅ Done |
| 9.6 | Brute Force Protection | Account lockout after N failures; exponential backoff | 🔴 Critical | M | 🟨 Partial |
| 9.7 | Secure Token Storage | Access token in memory; refresh token in HttpOnly cookie | 🔴 Critical | M | ✅ Done |
| 9.8 | Key Rotation | RSA key pair rotation without downtime | 🟡 High | L | ⬜ Todo |
| 9.9 | Dependency Audit | `cargo audit` + `npm audit` in CI | 🟡 High | S | ⬜ Todo |
| 9.10 | Penetration Testing | OWASP Top 10 checklist; automated scanning | 🟢 Medium | XL | ⬜ Todo |

---

## Phase 10 — Testing

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 10.1 | Backend Unit Tests | Auth logic, permission resolver, token validation | 🔴 Critical | L | ⬜ Todo |
| 10.2 | Backend Integration | DB integration tests with test containers | 🟡 High | L | ⬜ Todo |
| 10.3 | API E2E Tests | Full auth flows via `reqwest` test client | 🟡 High | L | ⬜ Todo |
| 10.4 | Frontend Unit Tests | Vitest; auth hooks, permission utils, form validation | 🟡 High | M | ⬜ Todo |
| 10.5 | Frontend E2E | Playwright; login, register, protected route, RBAC flows | 🟡 High | L | ⬜ Todo |
| 10.6 | Security Tests | Auth bypass attempts; token forgery; SQL injection | 🔴 Critical | XL | ⬜ Todo |
| 10.7 | Load Testing | k6 scripts; concurrent login, token refresh scenarios | 🟢 Medium | L | ⬜ Todo |

---

## Phase 11 — Documentation & Deployment

| # | Task | Description | Priority | Effort | Status |
|---|------|-------------|----------|--------|--------|
| 11.1 | API Docs | OpenAPI 3.0 spec; Swagger UI at `/api/docs` | 🟡 High | M | ⬜ Todo |
| 11.2 | README | Setup guide, env vars, quick start | 🔴 Critical | M | 🟨 Partial |
| 11.3 | Architecture Diagram | System design doc with auth flow diagrams | 🟡 High | M | ⬜ Todo |
| 11.4 | Deployment Guide | Docker, K8s manifests, environment config | 🟡 High | L | ⬜ Todo |
| 11.5 | SDK Generation | TypeScript SDK from OpenAPI spec | 🟢 Medium | M | ⬜ Todo |
| 11.6 | Developer Docs | Integration guide for third-party apps | 🟢 Medium | L | ⬜ Todo |

---

## Library Usage Map

| Library | Where Used | Purpose |
|---------|------------|---------|
| **Refine** | All pages | Auth provider, data provider, routing, CRUD hooks |
| **Mantine 5** | All UI | Components, styling, notifications, forms |
| **TanStack Table** | Users, Roles, Audit, Settings tables | Sorting, filtering, pagination, row selection |
| **TanStack Query** | Via Refine | Server state, caching, background refresh |
| **Next.js App Router** | All routes | File-based routing, route groups, layout nesting |

---

## Database Schema Overview

```sql
users, sessions, refresh_tokens,
email_verification_tokens, password_reset_tokens,
oauth_accounts, oauth_apps, authorization_codes,
roles, permissions, role_permissions, user_roles,
audit_logs, mfa_configs, backup_codes, login_history,
saml_providers, system_config
```

---

## Effort Legend

| Symbol | Meaning |
|--------|---------|
| S | Small (1-2 days) |
| M | Medium (3-4 days) |
| L | Large (1-2 weeks) |
| XL | Extra Large (2+ weeks) |

---

## Priority Legend

| Symbol | Meaning |
|--------|---------|
| 🔴 Critical | MVP blocker — must ship first |
| 🟡 High | Important — ship in sprint 1-2 |
| 🟢 Medium | Valuable — ship in sprint 3+ |

---

*AuthForge Build Plan v1.0 — Generated for full-stack auth platform implementation*
