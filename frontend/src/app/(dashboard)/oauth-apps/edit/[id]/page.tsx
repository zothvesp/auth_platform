"use client";

import { Group, Stack } from "@mantine/core";
import { useOne } from "@refinedev/core";
import { useParams, useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import {
  AppButton,
  AppSwitch,
  AppTextarea,
  AppTextInput,
  ErrorState,
  FormSkeleton,
  InlineAlert,
  PageHeader,
  SaveButton,
  SurfaceCard,
} from "@components/ui";
import type { OAuthAppRecord } from "@lib/admin-types";
import { apiRequest } from "@lib/request";
import { useTranslations } from "@lib/i18n";

export default function EditOAuthAppPage() {
  const t = useTranslations();
  const params = useParams<{ id: string }>();
  const router = useRouter();
  const query = useOne<OAuthAppRecord>({
    resource: "oauth_apps",
    id: params?.id ?? "",
  });
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [redirectUris, setRedirectUris] = useState("");
  const [allowedScopes, setAllowedScopes] = useState("");
  const [pkceRequired, setPkceRequired] = useState(false);
  const [isActive, setIsActive] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string>();
  const app = query.result;

  useEffect(() => {
    if (!app) return;
    setName(app.name);
    setDescription(app.description ?? "");
    setRedirectUris(app.redirectUris?.join("\n") ?? "");
    setAllowedScopes(app.allowedScopes?.join("\n") ?? "");
    setPkceRequired(app.pkceRequired);
    setIsActive(app.isActive);
  }, [app]);

  if (query.query.isLoading) return <FormSkeleton />;
  if (query.query.isError || !app)
    return <ErrorState title={t.pages.oauthApps.couldNotLoadEditor} />;

  const submit = async () => {
    setSaving(true);
    setError(undefined);
    try {
      await apiRequest(`/admin/oauth-apps/${app.id}`, {
        method: "PUT",
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
          is_active: isActive,
        }),
      });
      router.push(`/oauth-apps/show/${app.id}`);
      router.refresh();
    } catch (cause) {
      setError(
        typeof cause === "object" && cause && "message" in cause
          ? String(cause.message)
          : "Could not save OAuth app",
      );
    } finally {
      setSaving(false);
    }
  };

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.oauthApps.editTitle}
        description={app.name}
      />
      {error ? <InlineAlert tone="error">{error}</InlineAlert> : null}
      <SurfaceCard title={t.pages.oauthApps.appDetails}>
        <Stack p="md" spacing="md">
          <AppTextInput
            label={t.forms.name}
            disabled
            value={name}
          />
          <AppTextInput
            label={t.forms.description}
            value={description}
            onChange={(event) => setDescription(event.currentTarget.value)}
          />
          <AppTextarea
            label={t.forms.redirectUris}
            description={t.pages.oauthApps.redirectUrisHint}
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
          <AppSwitch
            label={t.forms.isActive}
            checked={isActive}
            onChange={(event) => setIsActive(event.currentTarget.checked)}
          />
        </Stack>
      </SurfaceCard>
      <Group position="right">
        <AppButton
          appVariant="secondary"
          onClick={() => router.push(`/oauth-apps/show/${app.id}`)}
        >
          {t.common.cancel}
        </AppButton>
        <SaveButton loading={saving} onClick={submit}>
          {t.pages.oauthApps.saveApp}
        </SaveButton>
      </Group>
    </Stack>
  );
}
