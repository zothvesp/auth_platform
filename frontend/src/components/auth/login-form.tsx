"use client";

import { Checkbox, Group, Stack } from "@mantine/core";
import { useLogin } from "@refinedev/core";
import { useState } from "react";
import {
  AppPasswordInput,
  AppTextInput,
  InlineAlert,
  SaveButton,
} from "@components/ui";
import { AuthLink, AuthShell } from "./auth-shell";

export const LoginForm = () => {
  const login = useLogin<{
    email: string;
    password: string;
    rememberMe: boolean;
  }>();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [rememberMe, setRememberMe] = useState(false);

  const errorMessage = login.error instanceof Error ? login.error.message : undefined;

  return (
    <AuthShell
      title="Sign in"
      description="Use your platform account credentials to continue."
      footer={
        <>
          New here? <AuthLink href="/register">Create an account</AuthLink>
        </>
      }
    >
      <form
        onSubmit={(event) => {
          event.preventDefault();
          login.mutate({ email, password, rememberMe });
        }}
      >
        <Stack spacing="md">
          {errorMessage ? <InlineAlert tone="error">{errorMessage}</InlineAlert> : null}
          <AppTextInput
            label="Email"
            type="email"
            autoComplete="email"
            required
            value={email}
            onChange={(event) => setEmail(event.currentTarget.value)}
          />
          <AppPasswordInput
            label="Password"
            autoComplete="current-password"
            required
            value={password}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          <Group position="apart">
            <Checkbox
              color="cyan"
              label="Remember this device"
              checked={rememberMe}
              onChange={(event) => setRememberMe(event.currentTarget.checked)}
            />
            <AuthLink href="/forgot-password">Forgot password?</AuthLink>
          </Group>
          <SaveButton loading={login.isPending}>Sign in</SaveButton>
        </Stack>
      </form>
    </AuthShell>
  );
};
