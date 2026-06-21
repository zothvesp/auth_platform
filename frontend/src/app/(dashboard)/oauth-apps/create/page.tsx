"use client";

import { Group, Stack } from "@mantine/core";
import { useRouter } from "next/navigation";
import { useState } from "react";
import {
  AppButton,
  AppSwitch,
  AppTextarea,
  AppTextInput,
  InlineAlert,
  PageHeader,
  SaveButton,
  SurfaceCard,
} from "@components/ui";
import { apiRequest } from "@lib/request";
import { useTranslations } from "@lib/i18n";

type CreateOAuthAppResponse = {
  id: string;
  clientId: string;
  clientSecret: string;
  name: string;
};

export default function CreateOAuthAppPage() {
  const t = useTranslations();
  const router = useRouter();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [redirectUris, setRedirectUris] = useState("");
  const [allowedScopes, setAllowedScopes] = useState("");
  const [pkceRequired, setPkceRequired] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string>();

  const submit = async () => {
    setSaving(true);
    setError(undefined);
    try {
      const app = await apiRequest<CreateOAuthAppResponse>("/admin/oauth-apps", {
        method: "POST",
        body: JSON.stringify({
          name,
          description,
          redirect_uris: redirectUris
            .split("\n")
            .map((u) => u.trim())
            .filter(Boolean),
          allowed_scopes: allowedScopes
            .split("\n")
            .map((s) => s.trim())
            .filter(Boolean),
          pkce_required: pkceRequired,
        }),
      });
      router.push(`/oauth-apps/show/${app.id}?client_secret=${encodeURIComponent(app.clientSecret)}`);
      router.refresh();
    } catch (cause) {
      setError(
        typeof cause === "object" && cause && "message" in cause
          ? String(cause.message)
          : "Could not create OAuth app",
      );
    } finally {
      setSaving(false);
    }
  };

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.oauthApps.createTitle}
        description={t.pages.oauthApps.createDesc}
      />
      {error ? <InlineAlert tone="error">{error}</InlineAlert> : null}
      <SurfaceCard title={t.pages.oauthApps.appDetails}>
        <Stack p="md" spacing="md">
          <AppTextInput
            label={t.forms.name}
            required
            value={name}
            onChange={(event) => setName(event.currentTarget.value)}
          />
          <AppTextInput
            label={t.forms.description}
            value={description}
            onChange={(event) => setDescription(event.currentTarget.value)}
          />
          <AppTextarea
            label={t.forms.redirectUris}
            description={t.pages.oauthApps.redirectUrisHint}
            required
            value={redirectUris}
            onChange={(event) => setRedirectUris(event.currentTarget.value)}
          />
          <AppTextarea
            label={t.forms.scopes}
            description={t.pages.oauthApps.scopesHint}
            value={allowedScopes}
            onChange={(event) => setAllowedScopes(event.currentTarget.value)}
          />
          <AppSwitch
            label={t.forms.pkceRequired}
            checked={pkceRequired}
            onChange={(event) => setPkceRequired(event.currentTarget.checked)}
          />
        </Stack>
      </SurfaceCard>
      <Group position="right">
        <AppButton appVariant="secondary" onClick={() => router.push("/oauth-apps")}>
          {t.common.cancel}
        </AppButton>
        <SaveButton loading={saving} onClick={submit}>
          {t.pages.oauthApps.createApp}
        </SaveButton>
      </Group>
    </Stack>
  );
}
