"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter, useSearchParams } from "next/navigation";
import { AppLoader, ErrorState } from "@components/ui";
import { AuthShell } from "@components/auth/auth-shell";
import { authApi } from "@lib/auth-api";
import { setAuthSession } from "@providers/auth-provider/auth-provider.client";

export default function OAuthCallbackPage() {
  const params = useParams<{ provider: string }>();
  const searchParams = useSearchParams();
  const router = useRouter();
  const [error, setError] = useState<string>();

  useEffect(() => {
    const provider = params?.provider;
    const code = searchParams?.get("code");
    const state = searchParams?.get("state");
    const providerError =
      searchParams?.get("error_description") ?? searchParams?.get("error");

    if (providerError) {
      setError(providerError);
      return;
    }
    if (!provider || !code || !state) {
      setError("The OAuth callback is missing required information.");
      return;
    }

    authApi
      .oauthCallback(provider, { code, state })
      .then((session) => {
        setAuthSession(session);
        router.replace("/");
        router.refresh();
      })
      .catch((cause) => {
        setError(cause instanceof Error ? cause.message : "OAuth sign-in failed");
      });
  }, [params?.provider, router, searchParams]);

  return (
    <AuthShell
      title={error ? "Sign-in failed" : "Completing sign-in"}
      description="Securely exchanging the provider authorization for your platform session."
    >
      {error ? (
        <ErrorState title="OAuth authentication failed" message={error} />
      ) : (
        <AppLoader label="Completing OAuth sign-in" />
      )}
    </AuthShell>
  );
}
