import { describe, it, expect } from "vitest";
import {
  decodeSession,
  encodeSession,
  decodePendingMfaLogin,
  encodePendingMfaLogin,
  type AuthSession,
  type PendingMfaLogin,
} from "./auth-api";

const makeSession = (overrides?: Partial<AuthSession>): AuthSession => ({
  tokens: {
    accessToken: "tok_abc123",
    tokenType: "Bearer",
    expiresIn: 3600,
    expiresAt: Date.now() + 3600_000,
  },
  user: {
    id: "u1",
    email: "test@example.com",
    displayName: "Test User",
    emailVerified: true,
    status: "active",
    mfaEnabled: false,
    authMethod: "password",
    createdAt: "2024-01-01T00:00:00Z",
    roles: [],
    permissions: [],
  },
  ...overrides,
});

describe("encodeSession / decodeSession", () => {
  it("roundtrips a valid session", () => {
    const session = makeSession();
    const encoded = encodeSession(session);
    expect(decodeSession(encoded)).toEqual(session);
  });

  it("returns null for undefined or null", () => {
    expect(decodeSession(undefined)).toBeNull();
    expect(decodeSession(null)).toBeNull();
  });

  it("returns null for malformed JSON", () => {
    expect(decodeSession("not-json")).toBeNull();
  });

  it("handles URI-encoded JSON", () => {
    const session = makeSession();
    const encoded = encodeURIComponent(JSON.stringify(session));
    expect(decodeSession(encoded)).toEqual(session);
  });
});

describe("encodePendingMfaLogin / decodePendingMfaLogin", () => {
  it("roundtrips valid pending MFA login", () => {
    const pending: PendingMfaLogin = {
      email: "user@example.com",
      password: "secret",
      expiresAt: Date.now() + 60_000,
      rememberMe: true,
    };
    const encoded = encodePendingMfaLogin(pending);
    expect(decodePendingMfaLogin(encoded)).toEqual(pending);
  });

  it("returns null for undefined or null", () => {
    expect(decodePendingMfaLogin(undefined)).toBeNull();
    expect(decodePendingMfaLogin(null)).toBeNull();
  });

  it("returns null for malformed JSON", () => {
    expect(decodePendingMfaLogin("bad")).toBeNull();
  });

  it("returns null when expired", () => {
    const pending: PendingMfaLogin = {
      email: "user@example.com",
      password: "secret",
      expiresAt: Date.now() - 1,
      rememberMe: false,
    };
    expect(decodePendingMfaLogin(encodePendingMfaLogin(pending))).toBeNull();
  });

  it("returns null when missing email", () => {
    const pending = { password: "s", expiresAt: Date.now() + 60_000, rememberMe: false } as unknown as PendingMfaLogin;
    expect(decodePendingMfaLogin(encodePendingMfaLogin(pending))).toBeNull();
  });

  it("returns null when missing password", () => {
    const pending = { email: "a@b.com", expiresAt: Date.now() + 60_000, rememberMe: false } as unknown as PendingMfaLogin;
    expect(decodePendingMfaLogin(encodePendingMfaLogin(pending))).toBeNull();
  });
});
