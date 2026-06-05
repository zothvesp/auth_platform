import { useState, useMemo, useRef, useEffect } from "react";

// ─── Data ────────────────────────────────────────────────────────────────────

const PHASES = [
  {
    id: 1,
    name: "Project Setup & Infrastructure",
    icon: "⚙️",
    color: "#6366f1",
    tasks: [
      { id: "1.1", name: "Monorepo Setup", desc: "pnpm/turborepo workspace; apps/frontend, apps/backend, packages/shared-types", priority: "critical", effort: "M", status: "todo" },
      { id: "1.2", name: "Frontend Scaffold", desc: "Vite + React 18 + TypeScript; path aliases configured", priority: "critical", effort: "S", status: "todo" },
      { id: "1.3", name: "Backend Scaffold", desc: "Cargo workspace; Axum + Tokio + SQLx; feature flags", priority: "critical", effort: "M", status: "todo" },
      { id: "1.4", name: "Database Setup", desc: "PostgreSQL schema; migrations via SQLx; seed data", priority: "critical", effort: "L", status: "todo" },
      { id: "1.5", name: "Docker Compose", desc: "postgres, redis, backend, frontend services for local dev", priority: "high", effort: "M", status: "todo" },
      { id: "1.6", name: "CI/CD Pipeline", desc: "GitHub Actions: lint, test, build, deploy on merge", priority: "high", effort: "L", status: "todo" },
      { id: "1.7", name: "Env Config", desc: ".env management; secrets vault integration points", priority: "high", effort: "S", status: "todo" },
      { id: "1.8", name: "Shared Types Package", desc: "TypeScript + Rust (via typeshare) type definitions", priority: "medium", effort: "M", status: "todo" },
    ]
  },
  {
    id: 2,
    name: "Core Authentication Backend",
    icon: "🔐",
    color: "#ef4444",
    tasks: [
      { id: "2.1", name: "User Model & DB", desc: "users table: id, email, password_hash, status, verified_at, created_at", priority: "critical", effort: "M", status: "todo" },
      { id: "2.2", name: "Password Hashing", desc: "Argon2id via argon2 crate; configurable work factor", priority: "critical", effort: "S", status: "todo" },
      { id: "2.3", name: "Registration Endpoint", desc: "POST /auth/register; validation, uniqueness check, email verification token", priority: "critical", effort: "M", status: "todo" },
      { id: "2.4", name: "Login Endpoint", desc: "POST /auth/login; credential validation, rate limiting, account lock", priority: "critical", effort: "M", status: "todo" },
      { id: "2.5", name: "JWT Implementation", desc: "Access token (15min) + refresh token (7d); RS256 signing; jsonwebtoken crate", priority: "critical", effort: "L", status: "todo" },
      { id: "2.6", name: "JWT Middleware", desc: "Axum extractor; validate signature, expiry, claims; attach user context", priority: "critical", effort: "M", status: "todo" },
      { id: "2.7", name: "Session-Based Auth", desc: "Redis-backed sessions; POST /auth/session; cookie with HttpOnly + Secure", priority: "high", effort: "L", status: "todo" },
      { id: "2.8", name: "Token Refresh", desc: "POST /auth/refresh; rotate refresh tokens; revoke old tokens", priority: "critical", effort: "M", status: "todo" },
      { id: "2.9", name: "Logout", desc: "POST /auth/logout; invalidate tokens/sessions; clear cookies", priority: "high", effort: "S", status: "todo" },
      { id: "2.10", name: "Email Verification", desc: "GET /auth/verify/:token; time-limited token; re-send endpoint", priority: "high", effort: "M", status: "todo" },
      { id: "2.11", name: "Password Reset", desc: "POST /auth/forgot-password + POST /auth/reset-password; HMAC token", priority: "high", effort: "M", status: "todo" },
      { id: "2.12", name: "Rate Limiting", desc: "tower-governor middleware; per-IP, per-user limits", priority: "critical", effort: "M", status: "todo" },
    ]
  },
  {
    id: 3,
    name: "OAuth 2.0 & SSO Backend",
    icon: "🔗",
    color: "#f59e0b",
    tasks: [
      { id: "3.1", name: "OAuth2 Framework", desc: "Generic OAuth2 client abstraction; state param; PKCE support", priority: "critical", effort: "L", status: "todo" },
      { id: "3.2", name: "Google OAuth", desc: "GET /auth/oauth/google + callback; user upsert", priority: "high", effort: "M", status: "todo" },
      { id: "3.3", name: "GitHub OAuth", desc: "GET /auth/oauth/github + callback", priority: "high", effort: "M", status: "todo" },
      { id: "3.4", name: "Microsoft/Azure AD", desc: "GET /auth/oauth/microsoft + callback; tenant config", priority: "high", effort: "L", status: "todo" },
      { id: "3.5", name: "OIDC/SSO Provider", desc: "OpenID Connect discovery; /.well-known/openid-configuration", priority: "high", effort: "L", status: "todo" },
      { id: "3.6", name: "SAML 2.0 SP", desc: "Service Provider impl; assertion parsing; samael crate", priority: "medium", effort: "XL", status: "todo" },
      { id: "3.7", name: "SAML IdP Metadata", desc: "GET /auth/saml/metadata; XML generation", priority: "medium", effort: "M", status: "todo" },
      { id: "3.8", name: "OAuth App Management", desc: "CRUD for OAuth apps; client_id/secret generation", priority: "medium", effort: "L", status: "todo" },
      { id: "3.9", name: "Token Introspection", desc: "POST /oauth/introspect; RFC 7662 compliance", priority: "medium", effort: "M", status: "todo" },
      { id: "3.10", name: "Token Revocation", desc: "POST /oauth/revoke; RFC 7009 compliance", priority: "medium", effort: "S", status: "todo" },
    ]
  },
  {
    id: 4,
    name: "RBAC & Permissions Backend",
    icon: "🛡️",
    color: "#10b981",
    tasks: [
      { id: "4.1", name: "Roles Table", desc: "roles: id, name, description, parent_role_id, is_system", priority: "critical", effort: "M", status: "todo" },
      { id: "4.2", name: "Permissions Table", desc: "permissions: id, resource, action, description", priority: "critical", effort: "M", status: "todo" },
      { id: "4.3", name: "Role-Permission Join", desc: "role_permissions many-to-many; inheritance via parent_role", priority: "critical", effort: "M", status: "todo" },
      { id: "4.4", name: "User-Role Join", desc: "user_roles with optional scope/context", priority: "critical", effort: "S", status: "todo" },
      { id: "4.5", name: "Permission Resolver", desc: "Recursive role hierarchy traversal; permission set computation", priority: "critical", effort: "L", status: "todo" },
      { id: "4.6", name: "RBAC Middleware", desc: "Axum guard extractor; require_permission! macro", priority: "critical", effort: "M", status: "todo" },
      { id: "4.7", name: "Roles API", desc: "GET/POST /roles, GET/PUT/DELETE /roles/:id", priority: "high", effort: "M", status: "todo" },
      { id: "4.8", name: "Permissions API", desc: "GET/POST /permissions, GET/PUT/DELETE /permissions/:id", priority: "high", effort: "M", status: "todo" },
      { id: "4.9", name: "Role Assignment API", desc: "POST/DELETE /users/:id/roles", priority: "high", effort: "S", status: "todo" },
      { id: "4.10", name: "Permission Check API", desc: "POST /auth/check-permission; batch permission checks", priority: "high", effort: "M", status: "todo" },
      { id: "4.11", name: "Default Roles Seed", desc: "Seed: super_admin, admin, manager, user, viewer", priority: "high", effort: "S", status: "todo" },
      { id: "4.12", name: "Audit Logging", desc: "audit_logs table; log all permission/role changes", priority: "medium", effort: "L", status: "todo" },
    ]
  },
  {
    id: 5,
    name: "Identity Management Backend",
    icon: "👤",
    color: "#8b5cf6",
    tasks: [
      { id: "5.1", name: "User Profile API", desc: "GET/PUT /users/me; avatar, display name, preferences", priority: "critical", effort: "M", status: "todo" },
      { id: "5.2", name: "User CRUD (Admin)", desc: "GET/PUT/DELETE /admin/users/:id; paginated list", priority: "high", effort: "M", status: "todo" },
      { id: "5.3", name: "User Search", desc: "GET /admin/users?q=&role=&status=; full-text search", priority: "high", effort: "M", status: "todo" },
      { id: "5.4", name: "Account Deactivation", desc: "Soft delete; PATCH /admin/users/:id/deactivate", priority: "high", effort: "S", status: "todo" },
      { id: "5.5", name: "Account Deletion", desc: "GDPR-compliant hard delete with data scrub", priority: "medium", effort: "M", status: "todo" },
      { id: "5.6", name: "MFA Setup", desc: "TOTP via totp-rs; POST /auth/mfa/setup, /verify, /disable", priority: "medium", effort: "XL", status: "todo" },
      { id: "5.7", name: "Backup Codes", desc: "Generate + validate one-time backup codes for MFA recovery", priority: "medium", effort: "M", status: "todo" },
      { id: "5.8", name: "Activity Log", desc: "Login history, IP, device info per user", priority: "medium", effort: "M", status: "todo" },
    ]
  },
  {
    id: 6,
    name: "Frontend Architecture",
    icon: "⚛️",
    color: "#06b6d4",
    tasks: [
      { id: "6.1", name: "TanStack Router Setup", desc: "File-based routing; route tree; lazy loading", priority: "critical", effort: "M", status: "todo" },
      { id: "6.2", name: "TanStack Query Setup", desc: "QueryClient config; devtools; error boundaries", priority: "critical", effort: "S", status: "todo" },
      { id: "6.3", name: "Auth Store", desc: "Zustand/TanStack Store: user, tokens, permissions; hydrate from storage", priority: "critical", effort: "M", status: "todo" },
      { id: "6.4", name: "API Client", desc: "Fetch wrapper; auto token attach; 401 interceptor; refresh flow", priority: "critical", effort: "L", status: "todo" },
      { id: "6.5", name: "Protected Route", desc: "beforeLoad guard; redirect to /login with redirect param", priority: "critical", effort: "M", status: "todo" },
      { id: "6.6", name: "Permission Guard", desc: "<PermissionGate permission='users:write'> HOC + hook", priority: "critical", effort: "M", status: "todo" },
      { id: "6.7", name: "TanStack Form", desc: "Form instances for login, register, profile, role forms", priority: "high", effort: "L", status: "todo" },
      { id: "6.8", name: "TanStack Table", desc: "Users table, roles table, audit logs with sorting/filtering", priority: "high", effort: "L", status: "todo" },
      { id: "6.9", name: "Error Boundary", desc: "Global + route-level error boundaries with fallback UI", priority: "high", effort: "M", status: "todo" },
      { id: "6.10", name: "Toast/Notification", desc: "Global notification system for auth events", priority: "high", effort: "S", status: "todo" },
    ]
  },
  {
    id: 7,
    name: "Frontend Auth Pages",
    icon: "🖥️",
    color: "#f97316",
    tasks: [
      { id: "7.1", name: "Login Page", desc: "Email/password + social login buttons; remember me", priority: "critical", effort: "M", status: "todo" },
      { id: "7.2", name: "Register Page", desc: "Full registration form with TanStack Form + validation", priority: "critical", effort: "M", status: "todo" },
      { id: "7.3", name: "Forgot Password", desc: "Email submission page", priority: "critical", effort: "S", status: "todo" },
      { id: "7.4", name: "Reset Password", desc: "Token-verified new password form", priority: "critical", effort: "S", status: "todo" },
      { id: "7.5", name: "Email Verification", desc: "Verify page + resend option", priority: "high", effort: "S", status: "todo" },
      { id: "7.6", name: "OAuth Callback", desc: "Handle provider redirects; exchange code; store tokens", priority: "critical", effort: "M", status: "todo" },
      { id: "7.7", name: "SSO Login", desc: "Enterprise SSO entry; domain-based provider discovery", priority: "high", effort: "M", status: "todo" },
      { id: "7.8", name: "MFA Prompt", desc: "TOTP code entry page; backup code fallback", priority: "medium", effort: "M", status: "todo" },
      { id: "7.9", name: "Session Expired", desc: "Auto-redirect with session expiry notification", priority: "high", effort: "S", status: "todo" },
      { id: "7.10", name: "Unauthorized (403)", desc: "Friendly 403 page with role/permission context", priority: "high", effort: "S", status: "todo" },
    ]
  },
  {
    id: 8,
    name: "Frontend Dashboard & Admin UI",
    icon: "📊",
    color: "#ec4899",
    tasks: [
      { id: "8.1", name: "Dashboard Layout", desc: "Sidebar nav; breadcrumbs; header with user menu", priority: "critical", effort: "L", status: "todo" },
      { id: "8.2", name: "Profile Page", desc: "Edit profile; change password; connected accounts", priority: "high", effort: "M", status: "todo" },
      { id: "8.3", name: "Users Admin Page", desc: "TanStack Table with search, filter, sort; bulk actions", priority: "high", effort: "L", status: "todo" },
      { id: "8.4", name: "User Detail Page", desc: "View/edit user; assign roles; activity log", priority: "high", effort: "M", status: "todo" },
      { id: "8.5", name: "Roles Admin Page", desc: "Create/edit/delete roles; permission matrix", priority: "high", effort: "L", status: "todo" },
      { id: "8.6", name: "Permissions Admin", desc: "View all permissions; grouped by resource", priority: "high", effort: "M", status: "todo" },
      { id: "8.7", name: "OAuth Apps Page", desc: "Register/manage OAuth applications", priority: "medium", effort: "M", status: "todo" },
      { id: "8.8", name: "Audit Log Page", desc: "TanStack Table for audit events; date range filter", priority: "medium", effort: "M", status: "todo" },
      { id: "8.9", name: "Security Settings", desc: "MFA setup; active sessions; trusted devices", priority: "medium", effort: "L", status: "todo" },
      { id: "8.10", name: "System Settings", desc: "SMTP config; OAuth provider config; SAML metadata", priority: "medium", effort: "XL", status: "todo" },
    ]
  },
  {
    id: 9,
    name: "Security & Hardening",
    icon: "🔒",
    color: "#dc2626",
    tasks: [
      { id: "9.1", name: "CORS Configuration", desc: "Strict origin allowlist; preflight handling", priority: "critical", effort: "S", status: "todo" },
      { id: "9.2", name: "CSP Headers", desc: "Content Security Policy; nonce-based script", priority: "critical", effort: "M", status: "todo" },
      { id: "9.3", name: "CSRF Protection", desc: "Double-submit cookie pattern for session auth", priority: "critical", effort: "M", status: "todo" },
      { id: "9.4", name: "Input Sanitization", desc: "Server-side: validator crate; XSS prevention", priority: "critical", effort: "M", status: "todo" },
      { id: "9.5", name: "SQL Injection Guard", desc: "Parameterized queries via SQLx (enforced by type system)", priority: "critical", effort: "S", status: "todo" },
      { id: "9.6", name: "Brute Force Protection", desc: "Account lockout after N failures; exponential backoff", priority: "critical", effort: "M", status: "todo" },
      { id: "9.7", name: "Secure Token Storage", desc: "Access token in memory; refresh token in HttpOnly cookie", priority: "critical", effort: "M", status: "todo" },
      { id: "9.8", name: "Key Rotation", desc: "RSA key pair rotation without downtime", priority: "high", effort: "L", status: "todo" },
      { id: "9.9", name: "Dependency Audit", desc: "cargo audit + npm audit in CI", priority: "high", effort: "S", status: "todo" },
      { id: "9.10", name: "Penetration Testing", desc: "OWASP Top 10 checklist; automated scanning", priority: "medium", effort: "XL", status: "todo" },
    ]
  },
  {
    id: 10,
    name: "Testing",
    icon: "🧪",
    color: "#84cc16",
    tasks: [
      { id: "10.1", name: "Backend Unit Tests", desc: "Auth logic, permission resolver, token validation", priority: "critical", effort: "L", status: "todo" },
      { id: "10.2", name: "Backend Integration", desc: "DB integration tests with test containers", priority: "high", effort: "L", status: "todo" },
      { id: "10.3", name: "API E2E Tests", desc: "Full auth flows via reqwest test client", priority: "high", effort: "L", status: "todo" },
      { id: "10.4", name: "Frontend Unit Tests", desc: "Vitest; auth hooks, permission utils, form validation", priority: "high", effort: "M", status: "todo" },
      { id: "10.5", name: "Frontend E2E", desc: "Playwright; login, register, protected route, RBAC flows", priority: "high", effort: "L", status: "todo" },
      { id: "10.6", name: "Security Tests", desc: "Auth bypass attempts; token forgery; SQL injection", priority: "critical", effort: "XL", status: "todo" },
      { id: "10.7", name: "Load Testing", desc: "k6 scripts; concurrent login, token refresh scenarios", priority: "medium", effort: "L", status: "todo" },
    ]
  },
  {
    id: 11,
    name: "Documentation & Deployment",
    icon: "📚",
    color: "#64748b",
    tasks: [
      { id: "11.1", name: "API Docs", desc: "OpenAPI 3.0 spec; Swagger UI at /api/docs", priority: "high", effort: "M", status: "todo" },
      { id: "11.2", name: "README", desc: "Setup guide, env vars, quick start", priority: "critical", effort: "M", status: "todo" },
      { id: "11.3", name: "Architecture Diagram", desc: "System design doc with auth flow diagrams", priority: "high", effort: "M", status: "todo" },
      { id: "11.4", name: "Deployment Guide", desc: "Docker, K8s manifests, environment config", priority: "high", effort: "L", status: "todo" },
      { id: "11.5", name: "SDK Generation", desc: "TypeScript SDK from OpenAPI spec", priority: "medium", effort: "M", status: "todo" },
      { id: "11.6", name: "Developer Docs", desc: "Integration guide for third-party apps", priority: "medium", effort: "L", status: "todo" },
    ]
  },
];

const TANSTACK_LIBS = [
  { name: "TanStack Router", color: "#ef4444", uses: ["Protected routes", "File-based routing", "Search params", "beforeLoad guards", "Lazy loading"] },
  { name: "TanStack Query", color: "#f59e0b", uses: ["Server state management", "Data fetching & caching", "Background refresh", "Optimistic updates", "Mutation handling"] },
  { name: "TanStack Table", color: "#10b981", uses: ["Users admin table", "Roles table", "Audit log table", "Sorting & filtering", "Pagination & row selection"] },
  { name: "TanStack Form", color: "#6366f1", uses: ["Login form", "Register form", "Profile edit form", "Role/permission forms", "Async validation"] },
  { name: "TanStack Virtual", color: "#8b5cf6", uses: ["Long permission lists", "Audit log virtualization", "Large user lists", "Virtualized dropdowns"] },
  { name: "TanStack Store", color: "#06b6d4", uses: ["Auth state (user, tokens)", "UI state management", "Permission cache", "Reactive global state"] },
];

const AUTH_FLOWS = [
  { name: "JWT", icon: "🎫", desc: "Access (15min) + Refresh (7d) tokens, RS256 signed, HttpOnly cookie" },
  { name: "Session", icon: "🍪", desc: "Redis-backed sessions, HttpOnly + Secure cookie, CSRF protected" },
  { name: "OAuth 2.0", icon: "🔗", desc: "Google, GitHub, Microsoft with PKCE flow" },
  { name: "OIDC/SSO", icon: "🏢", desc: "OpenID Connect with discovery document" },
  { name: "SAML 2.0", icon: "🔏", desc: "Enterprise SP with assertion parsing via samael" },
  { name: "MFA/TOTP", icon: "📱", desc: "Time-based one-time passwords + backup codes" },
];

const PRIORITY_CONFIG = {
  critical: { label: "Critical", color: "#ef4444", bg: "rgba(239,68,68,0.12)", dot: "#ef4444" },
  high: { label: "High", color: "#f59e0b", bg: "rgba(245,158,11,0.12)", dot: "#f59e0b" },
  medium: { label: "Medium", color: "#10b981", bg: "rgba(16,185,129,0.12)", dot: "#10b981" },
};

const EFFORT_CONFIG = {
  S: { label: "S", title: "Small (1-2d)", color: "#94a3b8" },
  M: { label: "M", title: "Medium (3-4d)", color: "#64748b" },
  L: { label: "L", title: "Large (1-2w)", color: "#475569" },
  XL: { label: "XL", title: "Extra Large (2w+)", color: "#334155" },
};

// ─── Main App ─────────────────────────────────────────────────────────────────

export default function AuthPlatformPlan() {
  const [tasks, setTasks] = useState(() => {
    const flat = {};
    PHASES.forEach(p => p.tasks.forEach(t => { flat[t.id] = t.status; }));
    return flat;
  });
  const [activePhase, setActivePhase] = useState(null);
  const [filterPriority, setFilterPriority] = useState("all");
  const [view, setView] = useState("phases"); // phases | tanstack | auth | overview
  const [expandedPhases, setExpandedPhases] = useState(new Set([1]));
  const [search, setSearch] = useState("");

  const cycleStatus = (taskId) => {
    setTasks(prev => {
      const current = prev[taskId];
      const next = current === "todo" ? "in-progress" : current === "in-progress" ? "done" : "todo";
      return { ...prev, [taskId]: next };
    });
  };

  const allTasks = useMemo(() => PHASES.flatMap(p => p.tasks.map(t => ({ ...t, status: tasks[t.id], phase: p.name, phaseColor: p.color }))), [tasks]);

  const stats = useMemo(() => {
    const total = allTasks.length;
    const done = allTasks.filter(t => tasks[t.id] === "done").length;
    const inProgress = allTasks.filter(t => tasks[t.id] === "in-progress").length;
    const critical = allTasks.filter(t => t.priority === "critical").length;
    const criticalDone = allTasks.filter(t => t.priority === "critical" && tasks[t.id] === "done").length;
    return { total, done, inProgress, critical, criticalDone, pct: Math.round((done / total) * 100) };
  }, [tasks, allTasks]);

  const togglePhase = (id) => {
    setExpandedPhases(prev => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  };

  const filteredPhases = useMemo(() => {
    return PHASES.map(p => ({
      ...p,
      tasks: p.tasks.filter(t => {
        const matchPriority = filterPriority === "all" || t.priority === filterPriority;
        const matchSearch = !search || t.name.toLowerCase().includes(search.toLowerCase()) || t.desc.toLowerCase().includes(search.toLowerCase());
        return matchPriority && matchSearch;
      }).map(t => ({ ...t, status: tasks[t.id] }))
    })).filter(p => p.tasks.length > 0);
  }, [filterPriority, search, tasks]);

  return (
    <div style={styles.root}>
      {/* Noise texture overlay */}
      <div style={styles.noiseOverlay} />

      {/* Header */}
      <header style={styles.header}>
        <div style={styles.headerLeft}>
          <div style={styles.logo}>
            <span style={styles.logoIcon}>⚡</span>
            <div>
              <div style={styles.logoName}>AuthForge</div>
              <div style={styles.logoSub}>Build Plan v1.0</div>
            </div>
          </div>
        </div>

        <nav style={styles.nav}>
          {[
            { key: "overview", label: "Overview" },
            { key: "phases", label: "Phases" },
            { key: "tanstack", label: "TanStack" },
            { key: "auth", label: "Auth Methods" },
          ].map(v => (
            <button
              key={v.key}
              onClick={() => setView(v.key)}
              style={{ ...styles.navBtn, ...(view === v.key ? styles.navBtnActive : {}) }}
            >
              {v.label}
            </button>
          ))}
        </nav>

        <div style={styles.headerRight}>
          <div style={styles.progressPill}>
            <span style={styles.progressNum}>{stats.pct}%</span>
            <span style={styles.progressLabel}>complete</span>
          </div>
        </div>
      </header>

      {/* Stats Bar */}
      <div style={styles.statsBar}>
        {[
          { label: "Total Tasks", value: stats.total, color: "#e2e8f0" },
          { label: "In Progress", value: stats.inProgress, color: "#f59e0b" },
          { label: "Completed", value: stats.done, color: "#10b981" },
          { label: "Critical Tasks", value: stats.critical, color: "#ef4444" },
          { label: "Critical Done", value: stats.criticalDone, color: "#ef4444" },
          { label: "Phases", value: PHASES.length, color: "#6366f1" },
        ].map(s => (
          <div key={s.label} style={styles.statItem}>
            <div style={{ ...styles.statValue, color: s.color }}>{s.value}</div>
            <div style={styles.statLabel}>{s.label}</div>
          </div>
        ))}
        <div style={styles.statItem}>
          <div style={styles.progressTrack}>
            <div style={{ ...styles.progressFill, width: `${stats.pct}%` }} />
          </div>
          <div style={styles.statLabel}>Overall Progress</div>
        </div>
      </div>

      {/* Main Content */}
      <main style={styles.main}>

        {/* ── OVERVIEW ── */}
        {view === "overview" && (
          <div style={styles.section}>
            <SectionHeader title="System Overview" sub="AuthForge — Full-stack Authentication & Identity Platform" />

            <div style={styles.overviewGrid}>
              <div style={styles.overviewCard}>
                <div style={styles.overviewCardTitle}>🏗️ Architecture</div>
                <ul style={styles.overviewList}>
                  <li><strong>Frontend:</strong> React 18 + Vite + TypeScript</li>
                  <li><strong>Backend:</strong> Rust + Axum + Tokio</li>
                  <li><strong>Database:</strong> PostgreSQL + SQLx migrations</li>
                  <li><strong>Cache/Sessions:</strong> Redis</li>
                  <li><strong>Routing:</strong> TanStack Router (file-based)</li>
                  <li><strong>State:</strong> TanStack Query + TanStack Store</li>
                  <li><strong>Forms:</strong> TanStack Form</li>
                  <li><strong>Tables:</strong> TanStack Table + Virtual</li>
                </ul>
              </div>

              <div style={styles.overviewCard}>
                <div style={styles.overviewCardTitle}>🔐 Auth Methods</div>
                <ul style={styles.overviewList}>
                  {AUTH_FLOWS.map(f => (
                    <li key={f.name}><strong>{f.icon} {f.name}:</strong> {f.desc}</li>
                  ))}
                </ul>
              </div>

              <div style={styles.overviewCard}>
                <div style={styles.overviewCardTitle}>🗄️ Database Tables</div>
                <div style={styles.dbTables}>
                  {["users", "sessions", "refresh_tokens", "email_verification_tokens", "password_reset_tokens", "oauth_accounts", "oauth_apps", "roles", "permissions", "role_permissions", "user_roles", "audit_logs", "mfa_configs", "backup_codes", "login_history"].map(t => (
                    <span key={t} style={styles.dbTable}>{t}</span>
                  ))}
                </div>
              </div>

              <div style={styles.overviewCard}>
                <div style={styles.overviewCardTitle}>📦 Key Rust Crates</div>
                <ul style={styles.overviewList}>
                  {["axum — HTTP framework", "tokio — async runtime", "sqlx — async DB driver", "jsonwebtoken — JWT RS256", "argon2 — password hashing", "tower-governor — rate limiting", "samael — SAML 2.0", "totp-rs — MFA/TOTP", "oauth2 — OAuth client", "serde — serialization", "validator — input validation", "redis — session store"].map(c => (
                    <li key={c}><code style={styles.code}>{c.split(" — ")[0]}</code>{" — " + c.split(" — ")[1]}</li>
                  ))}
                </ul>
              </div>

              <div style={styles.overviewCard}>
                <div style={styles.overviewCardTitle}>📦 Key npm Packages</div>
                <ul style={styles.overviewList}>
                  {["@tanstack/react-router — routing", "@tanstack/react-query — server state", "@tanstack/react-table — data tables", "@tanstack/react-form — forms", "@tanstack/react-virtual — virtualization", "@tanstack/store — global state", "zod — runtime validation", "jose — JWT client", "axios — HTTP client"].map(c => (
                    <li key={c}><code style={styles.code}>{c.split(" — ")[0]}</code>{" — " + c.split(" — ")[1]}</li>
                  ))}
                </ul>
              </div>

              <div style={styles.overviewCard}>
                <div style={styles.overviewCardTitle}>📋 Effort Legend</div>
                <ul style={styles.overviewList}>
                  <li><strong>S</strong> — Small (1–2 days)</li>
                  <li><strong>M</strong> — Medium (3–4 days)</li>
                  <li><strong>L</strong> — Large (1–2 weeks)</li>
                  <li><strong>XL</strong> — Extra Large (2+ weeks)</li>
                </ul>
                <div style={styles.overviewCardTitle} style={{ marginTop: 16, fontSize: 13, fontWeight: 700, color: "#94a3b8", textTransform: "uppercase", letterSpacing: "0.08em" }}>Priority</div>
                <ul style={styles.overviewList}>
                  <li><span style={{ color: "#ef4444" }}>🔴 Critical</span> — MVP blocker</li>
                  <li><span style={{ color: "#f59e0b" }}>🟡 High</span> — Sprint 1–2</li>
                  <li><span style={{ color: "#10b981" }}>🟢 Medium</span> — Sprint 3+</li>
                </ul>
                <div style={{ marginTop: 12, fontSize: 12, color: "#64748b" }}>
                  Click any task status dot to cycle: Todo → In Progress → Done
                </div>
              </div>
            </div>
          </div>
        )}

        {/* ── PHASES ── */}
        {view === "phases" && (
          <div style={styles.section}>
            <SectionHeader title="Build Phases" sub="Click a task's status indicator to update progress" />

            <div style={styles.toolbar}>
              <div style={styles.searchWrap}>
                <span style={styles.searchIcon}>⌕</span>
                <input
                  style={styles.searchInput}
                  placeholder="Search tasks..."
                  value={search}
                  onChange={e => setSearch(e.target.value)}
                />
              </div>
              <div style={styles.filterGroup}>
                {["all", "critical", "high", "medium"].map(p => (
                  <button
                    key={p}
                    onClick={() => setFilterPriority(p)}
                    style={{
                      ...styles.filterBtn,
                      ...(filterPriority === p ? {
                        background: p === "all" ? "#6366f1" : PRIORITY_CONFIG[p]?.dot || "#6366f1",
                        color: "#fff",
                      } : {})
                    }}
                  >
                    {p.charAt(0).toUpperCase() + p.slice(1)}
                  </button>
                ))}
              </div>
            </div>

            <div style={styles.phaseList}>
              {filteredPhases.map(phase => {
                const phaseTasks = phase.tasks;
                const donePct = Math.round((phaseTasks.filter(t => t.status === "done").length / phaseTasks.length) * 100);
                const expanded = expandedPhases.has(phase.id);

                return (
                  <div key={phase.id} style={{ ...styles.phaseCard, borderColor: phase.color + "40" }}>
                    <div style={styles.phaseHeader} onClick={() => togglePhase(phase.id)}>
                      <div style={styles.phaseHeaderLeft}>
                        <div style={{ ...styles.phaseIconWrap, background: phase.color + "20", color: phase.color }}>
                          {phase.icon}
                        </div>
                        <div>
                          <div style={styles.phaseName}>
                            <span style={styles.phaseNum}>Phase {phase.id}</span>
                            {phase.name}
                          </div>
                          <div style={styles.phaseMeta}>
                            {phaseTasks.length} tasks · {phaseTasks.filter(t => t.status === "done").length} done · {phaseTasks.filter(t => t.status === "in-progress").length} in progress
                          </div>
                        </div>
                      </div>
                      <div style={styles.phaseHeaderRight}>
                        <div style={styles.phaseProgress}>
                          <div style={styles.phaseProgressTrack}>
                            <div style={{ ...styles.phaseProgressFill, width: `${donePct}%`, background: phase.color }} />
                          </div>
                          <span style={{ ...styles.phaseProgressPct, color: phase.color }}>{donePct}%</span>
                        </div>
                        <span style={{ ...styles.chevron, transform: expanded ? "rotate(180deg)" : "rotate(0deg)" }}>▾</span>
                      </div>
                    </div>

                    {expanded && (
                      <div style={styles.taskTable}>
                        <div style={styles.taskTableHeader}>
                          <span style={{ width: 40 }}>ID</span>
                          <span style={{ flex: 1 }}>Task</span>
                          <span style={{ width: 90 }}>Priority</span>
                          <span style={{ width: 44 }}>Size</span>
                          <span style={{ width: 100 }}>Status</span>
                        </div>
                        {phaseTasks.map(task => (
                          <TaskRow key={task.id} task={task} onCycle={() => cycleStatus(task.id)} phaseColor={phase.color} />
                        ))}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* ── TANSTACK ── */}
        {view === "tanstack" && (
          <div style={styles.section}>
            <SectionHeader title="TanStack Library Usage" sub="Every @tanstack library mapped to AuthForge features" />
            <div style={styles.tanstackGrid}>
              {TANSTACK_LIBS.map(lib => (
                <div key={lib.name} style={{ ...styles.tanstackCard, borderTop: `3px solid ${lib.color}` }}>
                  <div style={{ ...styles.tanstackName, color: lib.color }}>{lib.name}</div>
                  <ul style={styles.tanstackList}>
                    {lib.uses.map(u => (
                      <li key={u} style={styles.tanstackItem}>
                        <span style={{ ...styles.tanstackDot, background: lib.color }} />
                        {u}
                      </li>
                    ))}
                  </ul>
                  <div style={styles.tanstackBadge}>
                    <a href={`https://tanstack.com`} target="_blank" style={{ ...styles.tanstackLink, color: lib.color }}>
                      tanstack.com ↗
                    </a>
                  </div>
                </div>
              ))}
            </div>

            <div style={styles.tanstackNote}>
              <div style={styles.tanstackNoteTitle}>Integration Architecture</div>
              <div style={styles.tanstackNoteBody}>
                <strong>Router + Query:</strong> Route loaders prefetch data via Query, keeping server state and navigation in sync.<br />
                <strong>Form + Query:</strong> Form mutations go through Query's <code style={styles.code}>useMutation</code> for optimistic updates and cache invalidation.<br />
                <strong>Table + Virtual:</strong> Tables with 1000+ rows use Virtual for windowed rendering — critical for audit logs and large user lists.<br />
                <strong>Store + Router:</strong> Auth state in TanStack Store drives Router's <code style={styles.code}>beforeLoad</code> guards for instant client-side protection.
              </div>
            </div>
          </div>
        )}

        {/* ── AUTH METHODS ── */}
        {view === "auth" && (
          <div style={styles.section}>
            <SectionHeader title="Authentication Methods" sub="All supported auth standards and their implementation details" />
            <div style={styles.authGrid}>
              {AUTH_FLOWS.map(flow => (
                <div key={flow.name} style={styles.authCard}>
                  <div style={styles.authIcon}>{flow.icon}</div>
                  <div style={styles.authName}>{flow.name}</div>
                  <div style={styles.authDesc}>{flow.desc}</div>
                </div>
              ))}
            </div>

            <div style={styles.authFlowDiagram}>
              <div style={styles.authFlowTitle}>JWT Flow</div>
              {["User submits credentials", "Server validates → issues access_token (15m) + refresh_token (7d)", "Access token stored in memory; refresh token in HttpOnly cookie", "API requests attach Bearer token in Authorization header", "On 401: interceptor uses refresh token to get new access token", "On logout: server revokes refresh token + clears cookie"].map((step, i) => (
                <div key={i} style={styles.flowStep}>
                  <div style={styles.flowStepNum}>{i + 1}</div>
                  <div style={styles.flowStepText}>{step}</div>
                </div>
              ))}
            </div>

            <div style={styles.authFlowDiagram}>
              <div style={styles.authFlowTitle}>OAuth 2.0 + PKCE Flow</div>
              {["Generate code_verifier (random) + code_challenge (SHA256)", "Redirect to provider with client_id, redirect_uri, scope, code_challenge", "Provider authenticates user → redirects back with authorization code", "Exchange code + code_verifier for access token (server-side)", "Fetch user profile from provider → upsert in users table", "Issue internal JWT → same session as password auth"].map((step, i) => (
                <div key={i} style={styles.flowStep}>
                  <div style={styles.flowStepNum}>{i + 1}</div>
                  <div style={styles.flowStepText}>{step}</div>
                </div>
              ))}
            </div>

            <div style={styles.authFlowDiagram}>
              <div style={styles.authFlowTitle}>RBAC Permission Check Flow</div>
              {["Request arrives with JWT/session", "Extract user_id from token/session", "Query user_roles → get role IDs", "Recursively traverse role hierarchy (parent_role_id)", "Aggregate all permissions from all roles in hierarchy", "Check required permission against set → Allow or 403"].map((step, i) => (
                <div key={i} style={styles.flowStep}>
                  <div style={styles.flowStepNum}>{i + 1}</div>
                  <div style={styles.flowStepText}>{step}</div>
                </div>
              ))}
            </div>
          </div>
        )}
      </main>

      <footer style={styles.footer}>
        AuthForge Build Plan · {PHASES.length} Phases · {PHASES.reduce((a, p) => a + p.tasks.length, 0)} Tasks · React + Rust + TanStack
      </footer>
    </div>
  );
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function SectionHeader({ title, sub }) {
  return (
    <div style={styles.sectionHeader}>
      <h2 style={styles.sectionTitle}>{title}</h2>
      <p style={styles.sectionSub}>{sub}</p>
    </div>
  );
}

function TaskRow({ task, onCycle, phaseColor }) {
  const statusConfig = {
    todo: { label: "Todo", color: "#475569", bg: "rgba(71,85,105,0.15)" },
    "in-progress": { label: "In Progress", color: "#f59e0b", bg: "rgba(245,158,11,0.15)" },
    done: { label: "Done", color: "#10b981", bg: "rgba(16,185,129,0.15)" },
  };
  const sc = statusConfig[task.status];
  const pc = PRIORITY_CONFIG[task.priority];

  return (
    <div style={styles.taskRow} title={task.desc}>
      <span style={{ ...styles.taskId, color: phaseColor }}>{task.id}</span>
      <div style={styles.taskName}>
        <span style={{ textDecoration: task.status === "done" ? "line-through" : "none", color: task.status === "done" ? "#475569" : "#e2e8f0" }}>
          {task.name}
        </span>
        <span style={styles.taskDesc}>{task.desc}</span>
      </div>
      <span style={{ ...styles.priorityBadge, color: pc.dot, background: pc.bg }}>
        {pc.label}
      </span>
      <span style={{ ...styles.effortBadge }} title={EFFORT_CONFIG[task.effort].title}>
        {task.effort}
      </span>
      <button onClick={onCycle} style={{ ...styles.statusBtn, color: sc.color, background: sc.bg }}>
        {sc.label}
      </button>
    </div>
  );
}

// ─── Styles ───────────────────────────────────────────────────────────────────

const styles = {
  root: {
    minHeight: "100vh",
    background: "#080c14",
    color: "#e2e8f0",
    fontFamily: "'DM Mono', 'IBM Plex Mono', 'Fira Code', monospace",
    position: "relative",
    overflow: "hidden",
  },
  noiseOverlay: {
    position: "fixed",
    inset: 0,
    backgroundImage: `url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.03'/%3E%3C/svg%3E")`,
    backgroundSize: "200px",
    pointerEvents: "none",
    zIndex: 0,
  },
  header: {
    position: "sticky",
    top: 0,
    zIndex: 100,
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "14px 28px",
    background: "rgba(8,12,20,0.92)",
    backdropFilter: "blur(20px)",
    borderBottom: "1px solid rgba(255,255,255,0.06)",
  },
  headerLeft: { display: "flex", alignItems: "center" },
  logo: { display: "flex", alignItems: "center", gap: 12 },
  logoIcon: { fontSize: 28, filter: "drop-shadow(0 0 8px #6366f1)" },
  logoName: { fontSize: 18, fontWeight: 700, color: "#f8fafc", letterSpacing: "-0.02em" },
  logoSub: { fontSize: 11, color: "#475569", letterSpacing: "0.1em", textTransform: "uppercase" },
  nav: { display: "flex", gap: 4 },
  navBtn: {
    background: "none", border: "1px solid transparent", color: "#64748b",
    padding: "6px 16px", borderRadius: 6, cursor: "pointer", fontSize: 13, fontFamily: "inherit",
    transition: "all 0.15s",
  },
  navBtnActive: { color: "#e2e8f0", borderColor: "rgba(255,255,255,0.12)", background: "rgba(255,255,255,0.06)" },
  headerRight: {},
  progressPill: {
    display: "flex", alignItems: "center", gap: 6,
    background: "rgba(99,102,241,0.15)", border: "1px solid rgba(99,102,241,0.3)",
    borderRadius: 20, padding: "4px 14px",
  },
  progressNum: { fontSize: 16, fontWeight: 700, color: "#818cf8" },
  progressLabel: { fontSize: 11, color: "#6366f1", textTransform: "uppercase", letterSpacing: "0.08em" },
  statsBar: {
    display: "flex", alignItems: "center", gap: 0,
    background: "rgba(255,255,255,0.02)", borderBottom: "1px solid rgba(255,255,255,0.05)",
    padding: "12px 28px", overflowX: "auto",
  },
  statItem: { minWidth: 100, padding: "0 20px", borderRight: "1px solid rgba(255,255,255,0.06)", textAlign: "center" },
  statValue: { fontSize: 22, fontWeight: 700, lineHeight: 1 },
  statLabel: { fontSize: 10, color: "#475569", textTransform: "uppercase", letterSpacing: "0.08em", marginTop: 4 },
  progressTrack: { height: 6, background: "rgba(255,255,255,0.08)", borderRadius: 3, width: 120, overflow: "hidden", margin: "0 auto" },
  progressFill: { height: "100%", background: "linear-gradient(90deg,#6366f1,#8b5cf6)", borderRadius: 3, transition: "width 0.5s" },
  main: { padding: "32px 28px", position: "relative", zIndex: 1 },
  section: { maxWidth: 1200, margin: "0 auto" },
  sectionHeader: { marginBottom: 32 },
  sectionTitle: { fontSize: 28, fontWeight: 700, color: "#f8fafc", margin: 0, letterSpacing: "-0.03em" },
  sectionSub: { fontSize: 14, color: "#475569", margin: "6px 0 0", fontStyle: "italic" },
  toolbar: { display: "flex", gap: 12, marginBottom: 20, alignItems: "center", flexWrap: "wrap" },
  searchWrap: { position: "relative", flex: 1, minWidth: 200 },
  searchIcon: { position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)", color: "#475569", fontSize: 18 },
  searchInput: {
    width: "100%", padding: "8px 12px 8px 36px", background: "rgba(255,255,255,0.04)",
    border: "1px solid rgba(255,255,255,0.08)", borderRadius: 8, color: "#e2e8f0",
    fontSize: 13, fontFamily: "inherit", outline: "none", boxSizing: "border-box",
  },
  filterGroup: { display: "flex", gap: 6 },
  filterBtn: {
    padding: "6px 14px", background: "rgba(255,255,255,0.04)", border: "1px solid rgba(255,255,255,0.08)",
    borderRadius: 20, color: "#94a3b8", fontSize: 12, cursor: "pointer", fontFamily: "inherit",
    transition: "all 0.15s",
  },
  phaseList: { display: "flex", flexDirection: "column", gap: 12 },
  phaseCard: {
    background: "rgba(255,255,255,0.025)", border: "1px solid",
    borderRadius: 12, overflow: "hidden",
  },
  phaseHeader: {
    display: "flex", alignItems: "center", justifyContent: "space-between",
    padding: "16px 20px", cursor: "pointer", userSelect: "none",
  },
  phaseHeaderLeft: { display: "flex", alignItems: "center", gap: 14 },
  phaseIconWrap: { width: 40, height: 40, borderRadius: 10, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 18 },
  phaseName: { fontSize: 15, fontWeight: 600, color: "#f1f5f9", display: "flex", alignItems: "center", gap: 8 },
  phaseNum: { fontSize: 11, color: "#475569", textTransform: "uppercase", letterSpacing: "0.08em", fontWeight: 400 },
  phaseMeta: { fontSize: 12, color: "#475569", marginTop: 2 },
  phaseHeaderRight: { display: "flex", alignItems: "center", gap: 16 },
  phaseProgress: { display: "flex", alignItems: "center", gap: 10 },
  phaseProgressTrack: { width: 80, height: 4, background: "rgba(255,255,255,0.08)", borderRadius: 2, overflow: "hidden" },
  phaseProgressFill: { height: "100%", borderRadius: 2, transition: "width 0.4s" },
  phaseProgressPct: { fontSize: 12, fontWeight: 700, minWidth: 30, textAlign: "right" },
  chevron: { fontSize: 18, color: "#475569", transition: "transform 0.2s", display: "block" },
  taskTable: { borderTop: "1px solid rgba(255,255,255,0.05)" },
  taskTableHeader: {
    display: "flex", alignItems: "center", gap: 12,
    padding: "8px 20px", background: "rgba(0,0,0,0.2)",
    fontSize: 10, color: "#475569", textTransform: "uppercase", letterSpacing: "0.1em",
  },
  taskRow: {
    display: "flex", alignItems: "flex-start", gap: 12,
    padding: "10px 20px", borderTop: "1px solid rgba(255,255,255,0.03)",
    transition: "background 0.1s", cursor: "default",
  },
  taskId: { width: 40, fontSize: 11, fontWeight: 700, flexShrink: 0, paddingTop: 2 },
  taskName: { flex: 1, display: "flex", flexDirection: "column", gap: 2 },
  taskDesc: { fontSize: 11, color: "#475569", lineHeight: 1.4 },
  priorityBadge: { width: 90, textAlign: "center", fontSize: 11, fontWeight: 600, padding: "2px 8px", borderRadius: 4, flexShrink: 0 },
  effortBadge: { width: 44, textAlign: "center", fontSize: 11, fontWeight: 700, color: "#64748b", flexShrink: 0, paddingTop: 2 },
  statusBtn: {
    width: 100, textAlign: "center", fontSize: 11, fontWeight: 600, padding: "3px 10px", borderRadius: 4,
    border: "none", cursor: "pointer", fontFamily: "inherit", flexShrink: 0,
    transition: "all 0.15s",
  },

  // Overview
  overviewGrid: { display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(340px, 1fr))", gap: 20 },
  overviewCard: { background: "rgba(255,255,255,0.03)", border: "1px solid rgba(255,255,255,0.07)", borderRadius: 12, padding: 24 },
  overviewCardTitle: { fontSize: 13, fontWeight: 700, color: "#94a3b8", textTransform: "uppercase", letterSpacing: "0.08em", marginBottom: 14 },
  overviewList: { margin: 0, paddingLeft: 18, display: "flex", flexDirection: "column", gap: 7, fontSize: 13, color: "#94a3b8", lineHeight: 1.5 },
  dbTables: { display: "flex", flexWrap: "wrap", gap: 6 },
  dbTable: { fontSize: 11, background: "rgba(99,102,241,0.12)", color: "#818cf8", border: "1px solid rgba(99,102,241,0.2)", borderRadius: 4, padding: "2px 8px" },
  code: { fontFamily: "inherit", background: "rgba(255,255,255,0.07)", borderRadius: 3, padding: "1px 5px", fontSize: "0.9em", color: "#a5b4fc" },

  // TanStack
  tanstackGrid: { display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))", gap: 20, marginBottom: 32 },
  tanstackCard: { background: "rgba(255,255,255,0.03)", border: "1px solid rgba(255,255,255,0.07)", borderRadius: 12, padding: 24, display: "flex", flexDirection: "column" },
  tanstackName: { fontSize: 16, fontWeight: 700, marginBottom: 16, letterSpacing: "-0.01em" },
  tanstackList: { margin: 0, padding: 0, listStyle: "none", display: "flex", flexDirection: "column", gap: 8, flex: 1 },
  tanstackItem: { fontSize: 13, color: "#94a3b8", display: "flex", alignItems: "center", gap: 8 },
  tanstackDot: { width: 6, height: 6, borderRadius: "50%", flexShrink: 0 },
  tanstackBadge: { marginTop: 16, paddingTop: 16, borderTop: "1px solid rgba(255,255,255,0.06)" },
  tanstackLink: { fontSize: 12, textDecoration: "none", fontWeight: 600 },
  tanstackNote: { background: "rgba(99,102,241,0.08)", border: "1px solid rgba(99,102,241,0.2)", borderRadius: 12, padding: 24 },
  tanstackNoteTitle: { fontSize: 14, fontWeight: 700, color: "#818cf8", marginBottom: 12 },
  tanstackNoteBody: { fontSize: 13, color: "#94a3b8", lineHeight: 1.8 },

  // Auth
  authGrid: { display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", gap: 16, marginBottom: 32 },
  authCard: { background: "rgba(255,255,255,0.03)", border: "1px solid rgba(255,255,255,0.07)", borderRadius: 12, padding: 24, textAlign: "center" },
  authIcon: { fontSize: 36, marginBottom: 12 },
  authName: { fontSize: 15, fontWeight: 700, color: "#f1f5f9", marginBottom: 8 },
  authDesc: { fontSize: 12, color: "#64748b", lineHeight: 1.6 },
  authFlowDiagram: { background: "rgba(255,255,255,0.02)", border: "1px solid rgba(255,255,255,0.06)", borderRadius: 12, padding: 24, marginBottom: 20 },
  authFlowTitle: { fontSize: 13, fontWeight: 700, color: "#818cf8", textTransform: "uppercase", letterSpacing: "0.08em", marginBottom: 16 },
  flowStep: { display: "flex", alignItems: "flex-start", gap: 14, marginBottom: 10 },
  flowStepNum: { width: 24, height: 24, borderRadius: "50%", background: "rgba(99,102,241,0.2)", color: "#818cf8", fontSize: 12, fontWeight: 700, display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0 },
  flowStepText: { fontSize: 13, color: "#94a3b8", lineHeight: 1.5, paddingTop: 3 },

  footer: { textAlign: "center", padding: "24px 0", color: "#334155", fontSize: 12, borderTop: "1px solid rgba(255,255,255,0.04)", marginTop: 40 },
};
