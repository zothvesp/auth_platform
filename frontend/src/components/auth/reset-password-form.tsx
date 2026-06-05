"use client";

import { Stack } from "@mantine/core";
import { useUpdatePassword } from "@refinedev/core";
import { useState } from "react";
import {
  AppPasswordInput,
  InlineAlert,
  PasswordStrengthMeter,
  SaveButton,
} from "@components/ui";
import { AuthLink, AuthShell } from "./auth-shell";

type ResetPasswordFormProps = {
  token?: string;
};

export const ResetPasswordForm = ({ token }: ResetPasswordFormProps) => {
  const updatePassword = useUpdatePassword<{ password: string; token: string }>();
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const mismatch = confirmPassword.length > 0 && password !== confirmPassword;
  const errorMessage =
    updatePassword.error instanceof Error ? updatePassword.error.message : undefined;

  return (
    <AuthShell
      title="Choose new password"
      description="Set a new password using the secure reset token from your email."
      footer={<AuthLink href="/login">Back to sign in</AuthLink>}
    >
      <form
        onSubmit={(event) => {
          event.preventDefault();
          if (!token || mismatch) return;
          updatePassword.mutate({ password, token });
        }}
      >
        <Stack spacing="md">
          {!token ? <InlineAlert tone="error">Missing reset token.</InlineAlert> : null}
          {errorMessage ? <InlineAlert tone="error">{errorMessage}</InlineAlert> : null}
          <AppPasswordInput
            label="New password"
            autoComplete="new-password"
            required
            value={password}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          <PasswordStrengthMeter value={password} />
          <AppPasswordInput
            label="Confirm password"
            autoComplete="new-password"
            required
            value={confirmPassword}
            error={mismatch ? "Passwords do not match" : undefined}
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
