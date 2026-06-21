"use client";

import { Stack } from "@mantine/core";
import { useRegister } from "@refinedev/core";
import { useState } from "react";
import {
  AppPasswordInput,
  AppTextInput,
  InlineAlert,
  PasswordStrengthMeter,
  SaveButton,
} from "@components/ui";
import { useTranslations } from "@lib/i18n";
import { AuthLink, AuthShell } from "./auth-shell";

export const RegisterForm = () => {
  const t = useTranslations();
  const register = useRegister<{
    displayName: string;
    email: string;
    password: string;
  }>();
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  const errorMessage = register.error instanceof Error ? register.error.message : undefined;

  return (
    <AuthShell
      title={t.auth.createAccount}
      description="Registration follows the backend password policy and creates a standard user account."
      footer={
        <>
          {t.auth.alreadyHaveAccount} <AuthLink href="/login">{t.auth.signInHere}</AuthLink>
        </>
      }
    >
      <form
        onSubmit={(event) => {
          event.preventDefault();
          register.mutate({ displayName, email, password });
        }}
      >
        <Stack spacing="md">
          {errorMessage ? <InlineAlert tone="error">{errorMessage}</InlineAlert> : null}
          <AppTextInput
            label={t.forms.displayName}
            required
            value={displayName}
            onChange={(event) => setDisplayName(event.currentTarget.value)}
          />
          <AppTextInput
            label={t.forms.email}
            type="email"
            autoComplete="email"
            required
            value={email}
            onChange={(event) => setEmail(event.currentTarget.value)}
          />
          <AppPasswordInput
            label={t.forms.password}
            autoComplete="new-password"
            required
            value={password}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          <PasswordStrengthMeter value={password} />
          <SaveButton loading={register.isPending}>{t.auth.createAccount}</SaveButton>
        </Stack>
      </form>
    </AuthShell>
  );
};
