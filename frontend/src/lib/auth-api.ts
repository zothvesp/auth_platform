import { apiRequest } from "./request";

export const API_URL =
  process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, "") ?? "http://localhost:8080/api/v1";

export type AuthUser = {
  id: string;
  email: string;
  displayName: string;
  avatarUrl?: string | null;
  emailVerified: boolean;
  status: string;
  mfaEnabled: boolean;
  authMethod: string;
  lastLoginAt?: string | null;
  createdAt: string;
  roles: Array<{
    id: string;
    name: string;
    description: string;
  }>;
  permissions: string[];
};

export type AuthTokens = {
  accessToken: string;
  tokenType: "Bearer";
  expiresIn: number;
  expiresAt: number;
};

export type AuthSession = {
  tokens: AuthTokens;
  user: AuthUser;
};

export type PublicConfig = {
  features: {
    allow_registration: boolean;
    mfa_enabled: boolean;
    oauth_github: boolean;
    oauth_google: boolean;
    oauth_microsoft: boolean;
    require_email_verification: boolean;
    saml_enabled: boolean;
  };
};

export { ApiError as AuthApiError } from "./request";

export type PendingMfaLogin = {
  email: string;
  expiresAt: number;
  password: string;
  rememberMe: boolean;
};

export const pendingMfaLoginKey = "auth_pending_mfa";

export const authApi = {
  getPublicConfig: () =>
    apiRequest<PublicConfig>("/config/public", { method: "GET" }),

  forgotPassword: (email: string) =>
    apiRequest<{ message: string }>("/auth/forgot-password", { method: "POST", body: { email } }),

  login: (body: {
    email: string;
    mfaCode?: string;
    password: string;
    rememberMe?: boolean;
  }) => apiRequest<AuthSession>("/auth/login", { method: "POST", body }),

  oauthCallback: (provider: string, body: { code: string; state: string }) =>
    apiRequest<AuthSession>(`/auth/oauth/${encodeURIComponent(provider)}/callback`, { method: "POST", body }),

  oauthStart: (provider: string) =>
    apiRequest<{ url: string }>(`/auth/oauth/${encodeURIComponent(provider)}`, { method: "GET" }),

  logout: (token?: string) =>
    apiRequest<void>("/auth/logout", { method: "POST", token }),

  register: (body: { displayName: string; email: string; password: string }) =>
    apiRequest<AuthSession>("/auth/register", { method: "POST", body }),

  resendVerification: (token: string) =>
    apiRequest<{ message: string }>("/auth/resend-verification", { method: "POST", body: {}, token }),

  resetPassword: (body: { password: string; token: string }) =>
    apiRequest<{ message: string }>("/auth/reset-password", { method: "POST", body }),

  verifyEmail: (token: string) =>
    apiRequest<{ message: string }>(`/auth/verify/${encodeURIComponent(token)}`, { method: "GET" }),
};

export type LinkedAccount = {
  provider: string;
  providerEmail?: string;
  createdAt: string;
};

export const oauthApi = {
  getLinkedAccounts: () =>
    apiRequest<LinkedAccount[]>("/users/me/oauth-accounts", { method: "GET" }),

  unlinkAccount: (provider: string) =>
    apiRequest<void>(`/users/me/oauth-accounts/${encodeURIComponent(provider)}`, {
      method: "DELETE",
    }),

  linkStart: (provider: string) =>
    apiRequest<{ url: string }>(`/auth/oauth/${encodeURIComponent(provider)}`, { method: "GET" }),
};

export const authSessionCookieName = "auth_session";

export const encodeSession = (session: AuthSession) => JSON.stringify(session);

export const decodeSession = (value?: string | null): AuthSession | null => {
  if (!value) return null;

  try {
    return JSON.parse(value) as AuthSession;
  } catch {
    try {
      return JSON.parse(decodeURIComponent(value)) as AuthSession;
    } catch {
      return null;
    }
  }
};

export const encodePendingMfaLogin = (value: PendingMfaLogin) => JSON.stringify(value);

export const decodePendingMfaLogin = (value?: string | null): PendingMfaLogin | null => {
  if (!value) return null;
  try {
    const pending = JSON.parse(value) as PendingMfaLogin;
    if (!pending.email || !pending.password || pending.expiresAt < Date.now()) {
      return null;
    }
    return pending;
  } catch {
    return null;
  }
};
