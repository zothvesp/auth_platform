"use client";

import type { AuthProvider } from "@refinedev/core";
import Cookies from "js-cookie";
import {
  authApi,
  AuthApiError,
  authSessionCookieName,
  decodePendingMfaLogin,
  decodeSession,
  encodePendingMfaLogin,
  encodeSession,
  pendingMfaLoginKey,
} from "@lib/auth-api";

const setSession = (session: Awaited<ReturnType<typeof authApi.login>>) => {
  sessionStorage.removeItem(pendingMfaLoginKey);
  Cookies.set(authSessionCookieName, encodeSession(session), {
    expires: new Date(session.tokens.expiresAt * 1000),
    path: "/",
    sameSite: "lax",
  });
};

const getSession = () => decodeSession(Cookies.get(authSessionCookieName));

const setPendingMfaLogin = (value: {
  email: string;
  password: string;
  rememberMe: boolean;
}) => {
  sessionStorage.setItem(
    pendingMfaLoginKey,
    encodePendingMfaLogin({
      ...value,
      expiresAt: Date.now() + 5 * 60 * 1000,
    }),
  );
};

export const authProviderClient: AuthProvider = {
  login: async ({ email, mfaCode, password, rememberMe }) => {
    try {
      const session = await authApi.login({
        email,
        mfaCode: mfaCode || undefined,
        password,
        rememberMe: Boolean(rememberMe),
      });
      setSession(session);
      return {
        success: true,
        redirectTo: "/",
      };
    } catch (error) {
      if (error instanceof AuthApiError && error.code === "MFA_REQUIRED") {
        setPendingMfaLogin({ email, password, rememberMe: Boolean(rememberMe) });
        return {
          success: true,
          redirectTo: "/mfa",
        };
      }

      return {
        success: false,
        error: {
          name: "LoginError",
          message: error instanceof Error ? error.message : "Invalid email or password",
        },
      };
    }
  },
  logout: async () => {
    const session = getSession();
    await authApi.logout(session?.tokens.accessToken).catch(() => undefined);
    sessionStorage.removeItem(pendingMfaLoginKey);
    Cookies.remove(authSessionCookieName, { path: "/" });
    return {
      success: true,
      redirectTo: "/login",
    };
  },
  check: async () => {
    const session = getSession();
    if (session?.tokens.accessToken) {
      return {
        authenticated: true,
      };
    }

    return {
      authenticated: false,
      logout: true,
      redirectTo: "/login",
    };
  },
  register: async ({ displayName, email, password }) => {
    try {
      const session = await authApi.register({ displayName, email, password });
      setSession(session);
      return {
        success: true,
        redirectTo: "/",
      };
    } catch (error) {
      return {
        success: false,
        error: {
          name: "RegisterError",
          message: error instanceof Error ? error.message : "Registration failed",
        },
      };
    }
  },
  forgotPassword: async ({ email }) => {
    try {
      await authApi.forgotPassword(email);
      return {
        success: true,
        redirectTo: "/login",
      };
    } catch (error) {
      return {
        success: false,
        error: {
          name: "ForgotPasswordError",
          message: error instanceof Error ? error.message : "Could not request reset email",
        },
      };
    }
  },
  updatePassword: async ({ password, token }) => {
    try {
      await authApi.resetPassword({ password, token });
      return {
        success: true,
        redirectTo: "/login",
      };
    } catch (error) {
      return {
        success: false,
        error: {
          name: "ResetPasswordError",
          message: error instanceof Error ? error.message : "Could not reset password",
        },
      };
    }
  },
  getPermissions: async () => {
    return getSession()?.user.permissions ?? null;
  },
  getIdentity: async () => {
    return getSession()?.user ?? null;
  },
  onError: async (error) => {
    if (error.response?.status === 401) {
      return {
        logout: true,
      };
    }

    return { error };
  },
};
