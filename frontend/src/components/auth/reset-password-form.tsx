"use client";

import { Stack } from "@mantine/core";
import { useUpdatePassword } from "@refinedev/core";
import { useRouter } from "next/navigation";
import { useState } from "react";
import {
  AppPasswordInput,
  InlineAlert,
  PasswordStrengthMeter,
  SaveButton,
} from "@components/ui";
import { useTranslations } from "@lib/i18n";
import { AuthLink, AuthShell } from "./auth-shell";

type ResetPasswordFormProps = {
  token?: string;
};

export const ResetPasswordForm = ({ token }: ResetPasswordFormProps) => {
  const t = useTranslations();
  const router = useRouter();
  const updatePassword = useUpdatePassword<{ password: string; token: string }>();
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const mismatch = confirmPassword.length > 0 && password !== confirmPassword;
  const errorMessage =
    updatePassword.error instanceof Error ? updatePassword.error.message : undefined;

  return (
    <AuthShell
      title={t.auth.chooseNewPasswordTitle}
      description={t.auth.chooseNewPasswordDesc}
      footer={<AuthLink href="/login">{t.auth.backToSignIn}</AuthLink>}
    >
      <form
        onSubmit={(event) => {
          event.preventDefault();
          if (!token || mismatch) return;
          updatePassword.mutate(
            { password, token },
            { onSuccess: () => router.push("/login") },
          );
        }}
      >
        <Stack spacing="md">
          {!token ? <InlineAlert tone="error">{t.auth.missingResetToken}</InlineAlert> : null}
          {errorMessage ? <InlineAlert tone="error">{errorMessage}</InlineAlert> : null}
          <AppPasswordInput
            label={t.forms.newPassword}
            autoComplete="new-password"
            required
            value={password}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          <PasswordStrengthMeter value={password} />
          <AppPasswordInput
            label={t.forms.confirmPassword}
            autoComplete="new-password"
            required
            value={confirmPassword}
            error={mismatch ? t.auth.passwordsDoNotMatch : undefined}
            onChange={(event) => setConfirmPassword(event.currentTarget.value)}
          />
          <SaveButton loading={updatePassword.isPending} disabled={!token || mismatch}>
            Update password
          </SaveButton>
        </Stack>
      </form>
    </AuthShell>
  );
};
