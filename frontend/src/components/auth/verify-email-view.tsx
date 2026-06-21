"use client";

import { Stack } from "@mantine/core";
import { useEffect, useState } from "react";
import { AppButton, AppLoader, InlineAlert } from "@components/ui";
import { authApi } from "@lib/auth-api";
import { useTranslations } from "@lib/i18n";
import { AuthLink, AuthShell } from "./auth-shell";

type VerifyEmailViewProps = {
  token?: string;
};

export const VerifyEmailView = ({ token }: VerifyEmailViewProps) => {
  const t = useTranslations();
  const [state, setState] = useState<"error" | "loading" | "success">(
    token ? "loading" : "error",
  );
  const [message, setMessage] = useState<string>(token ? t.auth.verifyingEmail : t.auth.missingVerificationToken);
  const [resending, setResending] = useState(false);
  const [resendResult, setResendResult] = useState<{ tone: "success" | "error"; message: string } | null>(null);

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
        setMessage(error instanceof Error ? error.message : t.auth.emailVerificationFailed);
      });
  }, [token, t]);

  const handleResend = () => {
    if (!token) return;
    setResending(true);
    setResendResult(null);
    authApi
      .resendVerification(token)
      .then(() => {
        setResendResult({ tone: "success", message: t.auth.verificationEmailResent });
      })
      .catch(() => {
        setResendResult({ tone: "error", message: t.auth.resendVerificationFailed });
      })
      .finally(() => setResending(false));
  };

  return (
    <AuthShell
      title={t.auth.emailVerificationTitle}
      description={t.auth.emailVerificationDesc}
      footer={<AuthLink href="/login">{t.auth.backToSignIn}</AuthLink>}
    >
      <Stack spacing="md">
        {state === "loading" ? <AppLoader label={message} /> : null}
        {state === "success" ? <InlineAlert tone="success">{message}</InlineAlert> : null}
        {state === "error" ? <InlineAlert tone="error">{message}</InlineAlert> : null}
        {state === "error" && token ? (
          <AppButton appVariant="secondary" loading={resending} onClick={handleResend}>
            {t.auth.resendVerification}
          </AppButton>
        ) : null}
        {resendResult ? <InlineAlert tone={resendResult.tone}>{resendResult.message}</InlineAlert> : null}
        {state !== "loading" ? <AuthLink href="/login">{t.auth.continueToSignIn}</AuthLink> : null}
      </Stack>
    </AuthShell>
  );
};
