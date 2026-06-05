"use client";

import { Stack, Text } from "@mantine/core";
import { useForgotPassword } from "@refinedev/core";
import { useState } from "react";
import { AppTextInput, InlineAlert, SaveButton } from "@components/ui";
import { AuthLink, AuthShell } from "./auth-shell";

export const ForgotPasswordForm = () => {
  const forgotPassword = useForgotPassword<{ email: string }>();
  const [email, setEmail] = useState("");
  const [submitted, setSubmitted] = useState(false);

  const errorMessage =
    forgotPassword.error instanceof Error ? forgotPassword.error.message : undefined;

  return (
    <AuthShell
      title="Reset password"
      description="Enter your email and we will send reset instructions if the account exists."
      footer={<AuthLink href="/login">Back to sign in</AuthLink>}
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
              If that email exists, a reset link has been sent.
            </InlineAlert>
          ) : null}
          {errorMessage ? <InlineAlert tone="error">{errorMessage}</InlineAlert> : null}
          <AppTextInput
            label="Email"
            type="email"
            autoComplete="email"
            required
            value={email}
            onChange={(event) => setEmail(event.currentTarget.value)}
          />
          <SaveButton loading={forgotPassword.isPending}>Send reset link</SaveButton>
          <Text size="xs" color="dimmed">
            For security, the backend returns the same response whether or not the email exists.
          </Text>
        </Stack>
      </form>
    </AuthShell>
  );
};
