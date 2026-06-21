"use client";

import { Stack, Text } from "@mantine/core";
import { useLogin } from "@refinedev/core";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { AppTextInput, InlineAlert, SaveButton } from "@components/ui";
import {
  decodePendingMfaLogin,
  pendingMfaLoginKey,
  type PendingMfaLogin,
} from "@lib/auth-api";
import { useTranslations } from "@lib/i18n";
import { AuthLink, AuthShell } from "./auth-shell";

export const MfaForm = () => {
  const t = useTranslations();
  const router = useRouter();
  const login = useLogin<PendingMfaLogin & { mfaCode: string }>();
  const [pending, setPending] = useState<PendingMfaLogin | null>(null);
  const [code, setCode] = useState("");

  useEffect(() => {
    const next = decodePendingMfaLogin(sessionStorage.getItem(pendingMfaLoginKey));
    setPending(next);
  }, []);

  const errorMessage = login.error instanceof Error ? login.error.message : undefined;

  return (
    <AuthShell
      title={t.auth.mfaTitle}
      description={t.auth.mfaDesc}
      footer={<AuthLink href="/login">{t.auth.useDifferentAccount}</AuthLink>}
    >
      <form
        onSubmit={(event) => {
          event.preventDefault();
          if (!pending) return;
          login.mutate(
            { ...pending, mfaCode: code },
            {
              onSuccess: (response) => {
                if (response.success) router.push(response.redirectTo ?? "/");
              },
            },
          );
        }}
      >
        <Stack spacing="md">
          {!pending ? (
            <InlineAlert tone="error">
              {t.auth.mfaExpired}
            </InlineAlert>
          ) : null}
          {errorMessage ? <InlineAlert tone="error">{errorMessage}</InlineAlert> : null}
          {pending ? (
            <Text size="sm" color="dimmed">
              {t.auth.completingMfaFor(pending.email)}
            </Text>
          ) : null}
          <AppTextInput
            label={t.forms.authCode}
            autoComplete="one-time-code"
            inputMode="numeric"
            maxLength={8}
            required
            value={code}
            onChange={(event) => setCode(event.currentTarget.value)}
          />
          <SaveButton loading={login.isPending} disabled={!pending || code.length < 6}>
            Verify code
          </SaveButton>
        </Stack>
      </form>
    </AuthShell>
  );
};
