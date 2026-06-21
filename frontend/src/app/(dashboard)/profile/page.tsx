"use client";

import { Group, Stack, Text } from "@mantine/core";
import Cookies from "js-cookie";
import { useEffect, useState } from "react";
import {
  AppButton,
  AppTextInput,
  ConfirmDialog,
  ErrorState,
  FormSkeleton,
  InlineAlert,
  PageHeader,
  SaveButton,
  StatusBadge,
  SurfaceCard,
} from "@components/ui";
import type { AuthUser, LinkedAccount } from "@lib/auth-api";
import {
  authSessionCookieName,
  decodeSession,
  encodeSession,
  oauthApi,
} from "@lib/auth-api";
import { useTranslations } from "@lib/i18n";
import { apiRequest } from "@lib/request";

export default function ProfilePage() {
  const t = useTranslations();
  const [user, setUser] = useState<AuthUser>();
  const [displayName, setDisplayName] = useState("");
  const [avatarUrl, setAvatarUrl] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string>();
  const [saved, setSaved] = useState(false);
  const [linkedAccounts, setLinkedAccounts] = useState<LinkedAccount[]>([]);
  const [providerToUnlink, setProviderToUnlink] = useState<string>();

  useEffect(() => {
    apiRequest<AuthUser>("/users/me")
      .then((data) => {
        setUser(data);
        setDisplayName(data.displayName);
        setAvatarUrl(data.avatarUrl ?? "");
      })
      .catch(() => setError(t.pages.profile.couldNotLoad))
      .finally(() => setLoading(false));
    oauthApi
      .getLinkedAccounts()
      .then(setLinkedAccounts)
      .catch(() => setLinkedAccounts([]));
  }, [t.pages.profile.couldNotLoad]);

  if (loading) return <FormSkeleton />;
  if (!user) return <ErrorState title={t.pages.profile.couldNotLoad} message={error} />;

  const submit = async () => {
    setSaving(true);
    setSaved(false);
    setError(undefined);
    try {
      const updated = await apiRequest<AuthUser>("/users/me", {
        method: "PUT",
        body: JSON.stringify({ displayName, avatarUrl: avatarUrl || null }),
      });
      const session = decodeSession(Cookies.get(authSessionCookieName));
      if (session) {
        Cookies.set(
          authSessionCookieName,
          encodeSession({ ...session, user: updated }),
          {
            expires: new Date(session.tokens.expiresAt * 1000),
            path: "/",
            sameSite: "lax",
          },
        );
      }
      setUser(updated);
      setSaved(true);
    } catch {
      setError(t.pages.profile.couldNotLoad);
    } finally {
      setSaving(false);
    }
  };

  const unlinkProvider = async () => {
    if (!providerToUnlink) return;
    setSaving(true);
    try {
      await oauthApi.unlinkAccount(providerToUnlink);
      setProviderToUnlink(undefined);
      const updated = await oauthApi.getLinkedAccounts();
      setLinkedAccounts(updated);
    } catch {
      setError(t.pages.profile.couldNotLoad);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.profile.title}
        description={t.pages.profile.description}
      />
      {saved ? <InlineAlert tone="success">{t.pages.profile.profileUpdated}</InlineAlert> : null}
      {error ? <InlineAlert tone="error">{error}</InlineAlert> : null}
      <SurfaceCard title={t.pages.profile.profileDetails}>
        <Stack p="md" spacing="md">
          <AppTextInput label={t.forms.email} value={user.email} disabled />
          <AppTextInput
            label={t.forms.displayName}
            required
            value={displayName}
            onChange={(event) => setDisplayName(event.currentTarget.value)}
          />
          <AppTextInput
            label={t.forms.avatarUrl}
            value={avatarUrl}
            onChange={(event) => setAvatarUrl(event.currentTarget.value)}
          />
          <Group position="right">
            <SaveButton loading={saving} onClick={submit}>
              {t.pages.profile.saveProfile}
            </SaveButton>
          </Group>
        </Stack>
      </SurfaceCard>

      <SurfaceCard
        title={t.pages.profile.connectedAccounts}
        description={t.pages.profile.connectedAccountsDesc}
      >
        <Stack p="md" spacing="md">
          {linkedAccounts.length > 0 ? (
            linkedAccounts.map((account) => (
              <Group key={account.provider} position="apart">
                <Group>
                  <StatusBadge value={account.provider} />
                  {account.providerEmail ? (
                    <Text size="sm" color="dimmed">
                      {account.providerEmail}
                    </Text>
                  ) : null}
                </Group>
                <AppButton
                  appVariant="danger"
                  loading={saving}
                  onClick={() => setProviderToUnlink(account.provider)}
                >
                  {t.pages.profile.disconnect}
                </AppButton>
              </Group>
            ))
          ) : (
            <Text size="sm" color="dimmed">
              {t.pages.profile.noAccountsConnected}
            </Text>
          )}
        </Stack>
      </SurfaceCard>

      <ConfirmDialog
        opened={Boolean(providerToUnlink)}
        title={t.pages.profile.disconnectAccount}
        tone="danger"
        confirmLabel={t.pages.profile.disconnect}
        loading={saving}
        message={t.pages.profile.disconnectConfirm(providerToUnlink ?? "")}
        onCancel={() => setProviderToUnlink(undefined)}
        onConfirm={unlinkProvider}
      />
    </Stack>
  );
}
