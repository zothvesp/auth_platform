"use client";

import { Stack } from "@mantine/core";
import { useEffect, useState } from "react";
import { AppLoader, InlineAlert } from "@components/ui";
import { authApi } from "@lib/auth-api";
import { AuthLink, AuthShell } from "./auth-shell";

type VerifyEmailViewProps = {
  token?: string;
};

export const VerifyEmailView = ({ token }: VerifyEmailViewProps) => {
  const [state, setState] = useState<"error" | "loading" | "success">(
    token ? "loading" : "error",
  );
  const [message, setMessage] = useState(token ? "Verifying email..." : "Missing verification token.");

  useEffect(() => {
    if (!token) return;

    authApi
      .verifyEmail(token)
      .then((response) => {
        setState("success");
        setMessage(response.message);
      })
      .catch((error) => {
        setState("error");
        setMessage(error instanceof Error ? error.message : "Email verification failed.");
      });
  }, [token]);

  return (
    <AuthShell
      title="Email verification"
      description="Confirming your account email address."
      footer={<AuthLink href="/login">Back to sign in</AuthLink>}
    >
      <Stack spacing="md">
        {state === "loading" ? <AppLoader label={message} /> : null}
        {state === "success" ? <InlineAlert tone="success">{message}</InlineAlert> : null}
        {state === "error" ? <InlineAlert tone="error">{message}</InlineAlert> : null}
        {state !== "loading" ? <AuthLink href="/login">Continue to sign in</AuthLink> : null}
      </Stack>
    </AuthShell>
  );
};
