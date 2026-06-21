"use client";

import { Stack } from "@mantine/core";
import { useState } from "react";
import { AppButton, AppTextInput, InlineAlert } from "@components/ui";
import { useTranslations } from "@lib/i18n";
import { API_URL } from "@lib/auth-api";
import { AuthLink, AuthShell } from "@components/auth/auth-shell";

export default function SsoPage() {
  const t = useTranslations();
  const [domain, setDomain] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(undefined);
    setLoading(true);

    try {
      const res = await fetch(
        `${API_URL}/auth/saml/${encodeURIComponent(domain)}/login`,
      );

      if (!res.ok) {
        const body = await res.json().catch(() => null);
        throw new Error(body?.message || t.auth.ssoProviderNotFound);
      }

      const { url } = await res.json();
      window.location.assign(url);
    } catch (err) {
      setError(err instanceof Error ? err.message : t.auth.ssoInitFailed);
    } finally {
      setLoading(false);
    }
  };

  return (
    <AuthShell
      title={t.auth.ssoTitle}
      description={t.auth.ssoDescription}
      footer={<AuthLink href="/login">{t.auth.backToSignIn}</AuthLink>}
    >
      <form onSubmit={handleSubmit}>
        <Stack spacing="md">
          {error ? <InlineAlert tone="error">{error}</InlineAlert> : null}
          <AppTextInput
            label={t.forms.email}
            placeholder="company.com"
            required
            value={domain}
            onChange={(e) => setDomain(e.currentTarget.value.trim())}
          />
          <AppButton type="submit" loading={loading}>
            {t.auth.ssoButton}
          </AppButton>
        </Stack>
      </form>
    </AuthShell>
  );
}
