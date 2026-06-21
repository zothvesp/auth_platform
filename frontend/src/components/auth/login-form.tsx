"use client";

import { Checkbox, Divider, Group, Stack } from "@mantine/core";
import { useLogin } from "@refinedev/core";
import { useEffect, useState } from "react";
import {
  AppButton,
  AppPasswordInput,
  AppTextInput,
  InlineAlert,
  SaveButton,
} from "@components/ui";
import { authApi, type PublicConfig } from "@lib/auth-api";
import { useTranslations } from "@lib/i18n";
import { AuthLink, AuthShell } from "./auth-shell";

export const LoginForm = () => {
  const t = useTranslations();
  const login = useLogin<{
    email: string;
    password: string;
    rememberMe: boolean;
  }>();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [rememberMe, setRememberMe] = useState(false);
  const [publicConfig, setPublicConfig] = useState<PublicConfig>();
  const [oauthLoading, setOauthLoading] = useState<string>();

  const errorMessage = login.error instanceof Error ? login.error.message : undefined;

  useEffect(() => {
    authApi.getPublicConfig().then(setPublicConfig).catch(() => undefined);
  }, []);

  const startOAuth = async (provider: string) => {
    setOauthLoading(provider);
    try {
      if (provider === "saml") {
        const response = await fetch(
          `${process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080/api/v1"}/auth/saml/default/login`,
        );
        const { url } = await response.json();
        window.location.assign(url);
      } else {
        const { url } = await authApi.oauthStart(provider);
        window.location.assign(url);
      }
    } finally {
      setOauthLoading(undefined);
    }
  };

  const oauthProviders = [
    { key: "google", label: "Google", enabled: publicConfig?.features.oauth_google },
    { key: "github", label: "GitHub", enabled: publicConfig?.features.oauth_github },
    {
      key: "microsoft",
      label: "Microsoft",
      enabled: publicConfig?.features.oauth_microsoft,
    },
  ].filter((provider) => provider.enabled);

  const showSaml = publicConfig?.features.saml_enabled;

  return (
    <AuthShell
      title={t.auth.signIn}
      description="Use your platform account credentials to continue."
      footer={
        <>
          {t.auth.newPasswordHere} <AuthLink href="/register">{t.auth.createAnAccount}</AuthLink>
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
            label={t.forms.email}
            type="email"
            autoComplete="email"
            required
            value={email}
            onChange={(event) => setEmail(event.currentTarget.value)}
          />
          <AppPasswordInput
            label={t.forms.password}
            autoComplete="current-password"
            required
            value={password}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          <Group position="apart">
            <Checkbox
              color="cyan"
              label={t.auth.rememberDevice}
              checked={rememberMe}
              onChange={(event) => setRememberMe(event.currentTarget.checked)}
            />
            <AuthLink href="/forgot-password">{t.auth.forgotPassword}</AuthLink>
          </Group>
          <SaveButton loading={login.isPending}>{t.auth.signIn}</SaveButton>
          {oauthProviders.length > 0 || showSaml ? (
            <>
              <Divider label={t.auth.orContinueWith} labelPosition="center" />
              <Group grow>
                {oauthProviders.map((provider) => (
                  <AppButton
                    key={provider.key}
                    appVariant="secondary"
                    loading={oauthLoading === provider.key}
                    disabled={Boolean(oauthLoading)}
                    onClick={() => startOAuth(provider.key)}
                  >
                    {provider.label}
                  </AppButton>
                ))}
                {showSaml ? (
                  <AppButton
                    appVariant="secondary"
                    loading={oauthLoading === "saml"}
                    disabled={Boolean(oauthLoading)}
                    onClick={() => startOAuth("saml")}
                  >
                    {t.auth.ssoButton}
                  </AppButton>
                ) : null}
              </Group>
            </>
          ) : null}
        </Stack>
      </form>
    </AuthShell>
  );
};
