import type { AuthProvider } from "@refinedev/core";
import { authSessionCookieName, decodeSession } from "@lib/auth-api";
import { cookies } from "next/headers";

export const authProviderServer: Pick<AuthProvider, "check"> = {
  check: async () => {
    const cookieStore = await cookies();
    const session = decodeSession(cookieStore.get(authSessionCookieName)?.value);

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
};
