"use client";

import { Stack, Text } from "@mantine/core";
import { useForgotPassword } from "@refinedev/core";
import { useState } from "react";
import { AppTextInput, InlineAlert, SaveButton } from "@components/ui";
import { useTranslations } from "@lib/i18n";
import { AuthLink, AuthShell } from "./auth-shell";

export const ForgotPasswordForm = () => {
  const t = useTranslations();
  const forgotPassword = useForgotPassword<{ email: string }>();
  const [email, setEmail] = useState("");
  const [submitted, setSubmitted] = useState(false);

  const errorMessage =
    forgotPassword.error instanceof Error ? forgotPassword.error.message : undefined;

  return (
    <AuthShell
      title={t.auth.resetPasswordTitle}
      description={t.auth.resetPasswordDesc}
      footer={<AuthLink href="/login">{t.auth.backToSignIn}</AuthLink>}
    >
      <form
        onSubmit={(event) => {
          event.preventDefault();
          forgotPassword.mutate({ email }, { onSuccess: () => setSubmitted(true) });
        }}
      >
        <Stack spacing="md">
          {submitted ? (
            <InlineAlert tone="success">
              {t.auth.resetLinkSent}
            </InlineAlert>
          ) : null}
          {errorMessage ? <InlineAlert tone="error">{errorMessage}</InlineAlert> : null}
          <AppTextInput
            label={t.forms.email}
            type="email"
            autoComplete="email"
            required
            value={email}
            onChange={(event) => setEmail(event.currentTarget.value)}
          />
          <SaveButton loading={forgotPassword.isPending}>Send reset link</SaveButton>
          <Text size="xs" color="dimmed">
            {t.auth.securityNote}
          </Text>
        </Stack>
      </form>
    </AuthShell>
  );
};
