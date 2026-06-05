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

type RequestOptions = {
  body?: unknown;
  method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  token?: string;
};

export class AuthApiError extends Error {
  code: string;
  status: number;

  constructor(message: string, status: number, code = "AUTH_ERROR") {
    super(message);
    this.name = "AuthApiError";
    this.code = code;
    this.status = status;
  }
}

export type PendingMfaLogin = {
  email: string;
  expiresAt: number;
  password: string;
  rememberMe: boolean;
};

export const pendingMfaLoginKey = "auth_pending_mfa";

const authRequest = async <T>(path: string, options: RequestOptions = {}) => {
  const response = await fetch(`${API_URL}${path}`, {
    method: options.method ?? "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      ...(options.token ? { Authorization: `Bearer ${options.token}` } : {}),
    },
    body: options.body ? JSON.stringify(options.body) : undefined,
  });

  if (!response.ok) {
    let message = "Request failed";
    let code = `HTTP_${response.status}`;

    try {
      const payload = await response.json();
      message = payload.message ?? payload.error ?? message;
      code = payload.code ?? code;
    } catch {
      message = response.statusText || message;
    }

    throw new AuthApiError(message, response.status, code);
  }

  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
};

export const authApi = {
  forgotPassword: (email: string) =>
    authRequest<{ message: string }>("/auth/forgot-password", {
      body: { email },
    }),

  login: (body: {
    email: string;
    mfaCode?: string;
    password: string;
    rememberMe?: boolean;
  }) => authRequest<AuthSession>("/auth/login", { body }),

  logout: (token?: string) =>
    authRequest<void>("/auth/logout", {
      method: "POST",
      token,
    }),

  register: (body: { displayName: string; email: string; password: string }) =>
    authRequest<AuthSession>("/auth/register", { body }),

  resendVerification: (token: string) =>
    authRequest<{ message: string }>("/auth/resend-verification", {
      body: {},
      token,
    }),

  resetPassword: (body: { password: string; token: string }) =>
    authRequest<{ message: string }>("/auth/reset-password", { body }),

  verifyEmail: (token: string) =>
    authRequest<{ message: string }>(`/auth/verify/${encodeURIComponent(token)}`, {
      method: "GET",
    }),
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
