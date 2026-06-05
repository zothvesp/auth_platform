# CLAUDE.md — AuthForge Backend Architecture Guide

> **Read this before touching any code.** This document is the single source of truth
> for architecture decisions, design principles, and contributor rules.

---

## 1. Design Principles (Non-Negotiable)

### DRY — Don't Repeat Yourself
Every piece of knowledge has **one authoritative location**. If you find yourself
writing the same SQL fragment, the same validation logic, or the same error mapping
in two places — stop. Find the shared abstraction first.

### GRASP — General Responsibility Assignment Software Patterns
- **Information Expert**: assign a responsibility to the type that has the data.  
  A `UserRepository` knows how to query users. A `RbacService` knows how to evaluate
  permissions. Neither knows about the other's internals.
- **Creator**: the thing that uses an object most is responsible for creating it.  
  `AppState` owns all repositories. Services receive repositories, never `PgPool`.
- **Controller**: route handlers are thin controllers — they translate HTTP ↔ domain.
  No business logic, no SQL, no crypto in handlers.
- **Low Coupling / High Cohesion**: repositories are cohesive around one aggregate.
  Services compose repositories; they don't embed queries.
- **Pure Fabrication**: `BaseRepository` is a pure fabrication — it doesn't map to a
  domain concept but exists to eliminate duplication across all concrete repos.

### YAGNI — You Aren't Gonna Need It
Do not add abstraction layers, generic parameters, or trait bounds unless they are
**needed right now** by existing code. Every layer must earn its place.

### KISS — Keep It Simple
Prefer flat over nested. Prefer concrete over generic. The simplest solution that
satisfies the stated requirement is the right solution.

---

## 2. Layer Architecture

```
HTTP Request
     │
     ▼
┌─────────────────────────────────────────────────────┐
│  Routes  (src/routes/)                              │
│  • Parse + validate request                         │
│  • Call one service method                          │
│  • Return HTTP response                             │
│  ✗ NO SQL  ✗ NO business logic  ✗ NO crypto         │
└──────────────────────┬──────────────────────────────┘
                       │ calls
                       ▼
┌─────────────────────────────────────────────────────┐
│  Services  (src/services/)                          │
│  • Orchestrate repositories                         │
│  • Own business rules (password hashing, JWT, etc.) │
│  • Own cross-cutting concerns (email, rate limiting)│
│  ✗ NO SQL  ✗ NO sqlx imports                        │
└──────────────────────┬──────────────────────────────┘
                       │ calls
                       ▼
┌─────────────────────────────────────────────────────┐
│  Repositories  (src/repositories/)                  │
│  • The ONLY place sqlx queries live                 │
│  • One repository per aggregate root                │
│  • All implement BaseRepository<Model, Id>          │
│  ✓ ONLY place with `sqlx::query*` calls             │
└──────────────────────┬──────────────────────────────┘
                       │ reads/writes
                       ▼
┌─────────────────────────────────────────────────────┐
│  Database  (Postgres via sqlx, Redis via redis-rs)  │
└─────────────────────────────────────────────────────┘
```

---

## 3. Repository Pattern Rules

### 3a. The Only Rule That Matters
**`sqlx::query*` calls exist in `src/repositories/` and nowhere else.**

If you see `sqlx::query` in `src/services/`, `src/routes/`, or `src/main.rs`
— that is a bug. Fix it before merging.

### 3b. BaseRepository Trait
Every concrete repository implements `BaseRepository<M, Id>` which provides:
- `find_by_id(id) → Option<M>`
- `find_all() → Vec<M>`  
- `delete(id) → ()`
- `exists(id) → bool`

Methods that don't fit the base (e.g. `find_by_email`) are defined directly on
the concrete repo as inherent methods. No trait gymnastics.

### 3c. One Aggregate Per Repository
| Repository | Aggregate Root | Owns |
|---|---|---|
| `UserRepository` | `User` | login_history, sessions (read) |
| `RoleRepository` | `Role` | role_permissions |
| `PermissionRepository` | `Permission` | — |
| `RefreshTokenRepository` | `RefreshToken` | — |
| `EmailTokenRepository` | `EmailVerificationToken`, `PasswordResetToken` | — |
| `OAuthRepository` | `OAuthAccount` | — |
| `AuditRepository` | `AuditLog` | login_history (write) |
| `SessionRepository` | `Session` | — |

### 3d. Repositories Receive `&PgPool`, Not `&AppState`
Services extract `&state.db.pool` before calling a repository. This keeps
repositories decoupled from the application wiring.

```rust
// ✓ Correct
async fn register(state: &AppState, ...) {
    let user_repo = UserRepository::new(&state.db.pool);
    user_repo.find_by_email(email).await?;
}

// ✗ Wrong — repo must not know about AppState
async fn find_by_email(state: &AppState, ...) { sqlx::query ... }
```

---

## 4. Do We Need Unit of Work?

**No. Not right now. Here's why (YAGNI).**

Unit of Work (UoW) provides two things:
1. **Transaction management** across multiple repositories in one atomic operation
2. **Change tracking** (à la Entity Framework)

### What we actually need
We need transactions. SQLx provides them without UoW overhead:

```rust
// In a service — clean, explicit, no ceremony
let mut tx = pool.begin().await?;
user_repo.create_with_tx(&mut tx, user).await?;
role_repo.assign_with_tx(&mut tx, user_id, role_id).await?;
tx.commit().await?;
```

Repositories accept `impl PgExecutor<'_>` so they work with both `&PgPool`
(auto-commit) and `&mut PgTransaction` (manual commit). This gives us transactional
safety **without** a UoW registry, an identity map, or dirty-flag tracking.

### When UoW would make sense
If we later needed:
- Cross-aggregate consistency with many repositories per request
- A domain event outbox pattern
- Entity change tracking for optimistic locking

At that point, introduce it. Not before.

### The pattern we use instead

```rust
// Repository methods accept PgExecutor — works for both pool and transaction
impl UserRepository {
    pub async fn create<'e, E: PgExecutor<'e>>(&self, exec: E, ...) -> AppResult<User> {
        sqlx::query_as!(...).fetch_one(exec).await
    }
}
```

---

## 5. Model Conventions

All models in `src/models/` follow this structure:

```rust
// models/base.rs — shared derives and trait
// Every model gets: Debug, Clone, Serialize, Deserialize, FromRow
// Every model that is a table row has: id: Uuid, created_at: DateTime<Utc>
// Every mutable table row also has: updated_at: DateTime<Utc>

pub trait Entity {
    fn id(&self) -> Uuid;
    fn created_at(&self) -> DateTime<Utc>;
}
```

Separate **row models** (what comes out of the DB) from **DTOs** (what goes over
the wire). DTOs live in `src/services/<name>.rs` alongside the service that
produces them — not in `models/`.

---

## 6. File Layout

```
src/
├── main.rs              # wiring only — no logic
├── config.rs            # env → typed config
├── state.rs             # AppState: holds pool + redis + config
├── error.rs             # AppError enum + IntoResponse
│
├── models/
│   ├── mod.rs
│   ├── base.rs          # Entity trait, shared derives macro
│   ├── user.rs
│   ├── role.rs
│   ├── permission.rs
│   ├── token.rs         # RefreshToken, EmailVerificationToken, PasswordResetToken
│   ├── session.rs
│   ├── oauth.rs
│   └── audit.rs
│
├── repositories/
│   ├── mod.rs
│   ├── base.rs          # BaseRepository trait + blanket helpers
│   ├── user.rs
│   ├── role.rs
│   ├── permission.rs
│   ├── token.rs
│   ├── session.rs
│   ├── oauth.rs
│   └── audit.rs
│
├── services/
│   ├── mod.rs
│   ├── auth.rs          # register, login, logout, refresh — DTOs live here
│   ├── rbac.rs          # permission evaluation, role management
│   ├── oauth.rs         # OAuth2/OIDC/SAML flows
│   ├── mfa.rs           # TOTP setup/verify
│   └── email.rs         # email dispatch (calls token repo internally)
│
└── routes/
    ├── mod.rs
    ├── auth.rs
    ├── users.rs
    ├── roles.rs
    ├── permissions.rs
    ├── oauth.rs
    └── admin.rs
```

---

## 7. Naming Conventions

| Layer | Suffix/Pattern | Example |
|---|---|---|
| Repository struct | `Repository` | `UserRepository` |
| Repository file | `repositories/<name>.rs` | `repositories/user.rs` |
| Service functions | plain verb | `auth::login(...)` |
| Route handlers | plain verb (HTTP verb implied) | `register(...)` |
| DTOs | `Dto` or `Response`/`Request` suffix | `UserDto`, `LoginRequest` |
| DB row models | plain noun, no suffix | `User`, `Role` |

---

## 8. Testing Rules

- Services are tested with a real test DB (sqlx test containers).
- Repositories are tested directly — no mocking.
- Route handlers are tested via `axum-test` HTTP client.
- No `mockall` for repositories — the overhead of mocking exceeds the benefit
  when a real PG test instance boots in < 1s via Docker.

---

## 9. What This Document Is Not

This is not a style guide. Formatting is handled by `rustfmt`.  
This is not a security checklist. That lives in `SECURITY.md`.  
This is the **architecture contract** — the rules the codebase enforces.

---

## 10. Config Architecture (DB-First, Env-Fallback)

### Rule: tunables live in `system_config`, secrets live in env

| Setting type | Where it lives | Why |
|---|---|---|
| JWT private/public key | `AppConfig` (env only) | Cannot be stored in DB — needed before DB is ready |
| SMTP password | `AppConfig` (env only) | Secret — never in DB |
| OAuth client secrets | `AppConfig` (env only) | Secret — never in DB |
| Password policy | `system_config` table | UI-visible, admin-editable at runtime |
| JWT expiry times | `system_config` table (env fallback) | Tunable without deploy |
| Max login attempts | `system_config` table (env fallback) | Tunable without deploy |
| Feature flags | `system_config` table | Admin-editable UI flags |
| Field length limits | `system_config` table | UI-visible validation rules |

### Rule: services use `services::config::*`, never `state.config.*` for tunables

```rust
// ✗ Wrong — bypasses DB override
let expiry = state.config.jwt_access_expiry_secs;

// ✓ Correct — reads DB first, falls back to env value
let expiry = services::config::jwt_access_expiry_secs(state).await;
```

### Frontend config flow

1. App boots → `GET /api/v1/config/public` (no auth)
2. Response cached in TanStack Query for 5 minutes
3. Zod schemas built from `PublicConfig.password_policy` + `PublicConfig.validation_rules`
4. Feature flags gate OAuth buttons, SSO link, registration link
5. No validation rules or permission strings hardcoded in frontend

---

## 11. Permission Architecture

### Server is the authority — always

- The JWT contains the **fully resolved, hierarchy-expanded** permission set
- `super_admin` bypass logic exists **only on the server** (in the RBAC service)
- The frontend reads `user.permissions[]` directly from the JWT claims
- No client-side role hierarchy evaluation
- No hardcoded `if role === 'super_admin'` in frontend code

### Permission key constants

Defined once in:
- **Backend**: `models/config.rs` → `keys` module (config keys)
- **Frontend**: `utils/permissions.ts` → `P` and `R` constants

These must stay in sync with the migration seed data. The comment in
`utils/permissions.ts` documents which migration file to check.
