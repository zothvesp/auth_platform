"use client";

import { Anchor, Divider, Group, Stack, Text } from "@mantine/core";
import type { ColumnDef } from "@tanstack/react-table";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  AppButton,
  AppPasswordInput,
  AppTextInput,
  CodeBlock,
  ConfirmDialog,
  DataTable,
  DateTimeText,
  InlineAlert,
  PageHeader,
  SaveButton,
  StatusBadge,
  SurfaceCard,
} from "@components/ui";
import type { AuthUser, LinkedAccount, PublicConfig } from "@lib/auth-api";
import { authApi, oauthApi } from "@lib/auth-api";
import { useTranslations } from "@lib/i18n";
import { apiRequest } from "@lib/request";

type LoginHistoryRecord = {
  auth_method: string;
  created_at: string;
  id: string;
  ip_address: string;
  location?: string | null;
  success: boolean;
  user_agent: string;
};

type SessionRecord = {
  created_at: string;
  expires_at: string;
  id: string;
  ip_address: string;
  user_agent: string;
};

type MfaSetup = {
  qr_code: string;
  secret: string;
};

export default function SecurityPage() {
  const t = useTranslations();
  const [user, setUser] = useState<AuthUser>();
  const [history, setHistory] = useState<LoginHistoryRecord[]>([]);
  const [sessions, setSessions] = useState<SessionRecord[]>([]);
  const [linkedAccounts, setLinkedAccounts] = useState<LinkedAccount[]>([]);
  const [publicConfig, setPublicConfig] = useState<PublicConfig>();
  const [providerToUnlink, setProviderToUnlink] = useState<string>();
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [mfaSetup, setMfaSetup] = useState<MfaSetup>();
  const [mfaCode, setMfaCode] = useState("");
  const [useBackupCode, setUseBackupCode] = useState(false);
  const [backupCodes, setBackupCodes] = useState<string[]>([]);
  const [sessionToRevoke, setSessionToRevoke] = useState<SessionRecord>();
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ tone: "error" | "success"; text: string }>();

  const loadSessions = useCallback(() => {
    return apiRequest<SessionRecord[]>("/users/me/sessions")
      .then(setSessions)
      .catch(() => setSessions([]));
  }, []);

  const loadLinkedAccounts = useCallback(() => {
    return oauthApi.getLinkedAccounts().then(setLinkedAccounts).catch(() => setLinkedAccounts([]));
  }, []);

  useEffect(() => {
    void Promise.all([
      apiRequest<AuthUser>("/users/me").then(setUser),
      apiRequest<LoginHistoryRecord[]>("/users/me/login-history").then(setHistory),
      loadSessions(),
      loadLinkedAccounts(),
      authApi.getPublicConfig().then(setPublicConfig).catch(() => undefined),
    ]);
  }, [loadSessions, loadLinkedAccounts]);

  const historyColumns = useMemo<ColumnDef<LoginHistoryRecord>[]>(
    () => [
      {
        accessorKey: "created_at",
        header: t.pages.security.time,
        cell: ({ row }) => <DateTimeText value={row.original.created_at} />,
      },
      { accessorKey: "ip_address", header: t.pages.security.ipAddress },
      { accessorKey: "auth_method", header: t.pages.security.method },
      {
        accessorKey: "success",
        header: t.table.columns.result,
        cell: ({ row }) => <StatusBadge value={row.original.success ? "success" : "failed"} />,
      },
      {
        accessorKey: "user_agent",
        header: t.pages.security.device,
        cell: ({ row }) => (
          <Text size="xs" lineClamp={2}>
            {row.original.user_agent}
          </Text>
        ),
      },
    ],
    [t.pages.security.time, t.pages.security.ipAddress, t.pages.security.method, t.table.columns.result, t.pages.security.device],
  );

  const sessionColumns = useMemo<ColumnDef<SessionRecord>[]>(
    () => [
      {
        accessorKey: "created_at",
        header: t.pages.security.started,
        cell: ({ row }) => <DateTimeText value={row.original.created_at} />,
      },
      {
        accessorKey: "expires_at",
        header: t.pages.security.expires,
        cell: ({ row }) => <DateTimeText value={row.original.expires_at} />,
      },
      { accessorKey: "ip_address", header: t.pages.security.ipAddress },
      {
        accessorKey: "user_agent",
        header: t.pages.security.device,
        cell: ({ row }) => (
          <Text size="xs" lineClamp={2}>
            {row.original.user_agent}
          </Text>
        ),
      },
      {
        id: "actions",
        header: "",
        cell: ({ row }) => (
          <AppButton appVariant="danger" onClick={() => setSessionToRevoke(row.original)}>
            {t.pages.security.revoke}
          </AppButton>
        ),
      },
    ],
    [t.pages.security.started, t.pages.security.expires, t.pages.security.ipAddress, t.pages.security.device, t.pages.security.revoke],
  );

  const runAction = async (action: () => Promise<void>, success: string) => {
    setSaving(true);
    setMessage(undefined);
    try {
      await action();
      setMessage({ tone: "success", text: success });
    } catch {
      setMessage({ tone: "error", text: t.pages.security.securityError });
    } finally {
      setSaving(false);
    }
  };

  const changePassword = () =>
    runAction(async () => {
      await apiRequest("/users/me/change-password", {
        method: "POST",
        body: JSON.stringify({ currentPassword, newPassword }),
      });
      setCurrentPassword("");
      setNewPassword("");
    }, t.pages.security.passwordChanged);

  const beginMfaSetup = () =>
    runAction(async () => {
      const setup = await apiRequest<MfaSetup>("/auth/mfa/setup", { method: "POST" });
      setMfaSetup(setup);
      setBackupCodes([]);
    }, t.pages.security.mfaSetupCreated);

  const enableMfa = () =>
    runAction(async () => {
      const result = await apiRequest<{ backup_codes: string[] }>("/auth/mfa/verify", {
        method: "POST",
        body: JSON.stringify({ code: mfaCode }),
      });
      setBackupCodes(result.backup_codes);
      setMfaCode("");
      setMfaSetup(undefined);
      setUser((current) => (current ? { ...current, mfaEnabled: true } : current));
    }, t.pages.security.mfaEnabledSuccess);

  const disableMfa = () =>
    runAction(async () => {
      await apiRequest("/auth/mfa/disable", {
        method: "POST",
        body: JSON.stringify({ code: mfaCode }),
      });
      setMfaCode("");
      setBackupCodes([]);
      setUser((current) => (current ? { ...current, mfaEnabled: false } : current));
    }, t.pages.security.mfaDisabled);

  const revokeSession = async () => {
    if (!sessionToRevoke) return;
    await runAction(async () => {
      await apiRequest(`/users/me/sessions/${sessionToRevoke.id}`, { method: "DELETE" });
      setSessionToRevoke(undefined);
      await loadSessions();
    }, t.pages.security.sessionRevoked);
  };

  const unlinkProvider = async () => {
    if (!providerToUnlink) return;
    await runAction(async () => {
      await oauthApi.unlinkAccount(providerToUnlink);
      setProviderToUnlink(undefined);
      await loadLinkedAccounts();
    }, `${providerToUnlink} account disconnected.`);
  };

  const linkProvider = async (provider: string) => {
    try {
      const { url } = await oauthApi.linkStart(provider);
      window.location.assign(url);
    } catch {
      setMessage({ tone: "error", text: `Could not start ${provider} linking.` });
    }
  };

  const oauthProviders = [
    { key: "google", label: "Google", enabled: publicConfig?.features.oauth_google },
    { key: "github", label: "GitHub", enabled: publicConfig?.features.oauth_github },
    { key: "microsoft", label: "Microsoft", enabled: publicConfig?.features.oauth_microsoft },
  ].filter((p) => p.enabled);

  const linkedProviders = new Set(linkedAccounts.map((a) => a.provider));
  const unlinkedProviders = oauthProviders.filter((p) => !linkedProviders.has(p.key));

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.security.title}
        description={t.pages.security.description}
      />
      {message ? <InlineAlert tone={message.tone}>{message.text}</InlineAlert> : null}
      <SurfaceCard title={t.pages.security.changePassword}>
        <Stack p="md" spacing="md">
          <AppPasswordInput
            label={t.pages.security.currentPassword}
            required
            value={currentPassword}
            onChange={(event) => setCurrentPassword(event.currentTarget.value)}
          />
          <AppPasswordInput
            label={t.pages.security.newPassword}
            required
            value={newPassword}
            onChange={(event) => setNewPassword(event.currentTarget.value)}
          />
          <Group position="right">
            <SaveButton loading={saving} onClick={changePassword}>
              {t.pages.security.changePassword}
            </SaveButton>
          </Group>
        </Stack>
      </SurfaceCard>

      <SurfaceCard
        title={t.pages.security.mfaEnabled}
        description={t.pages.security.mfaDesc}
        action={<StatusBadge value={user?.mfaEnabled ? t.common.enabled : t.common.disabled} />}
      >
        <Stack p="md" spacing="md">
          {!user?.mfaEnabled && !mfaSetup ? (
            <AppButton loading={saving} onClick={beginMfaSetup}>
              {t.pages.security.setupMfa}
            </AppButton>
          ) : null}
          {mfaSetup ? (
            <>
              <InlineAlert>
                {t.pages.security.mfaSetupInfo}
              </InlineAlert>
              <CodeBlock language="TOTP secret" value={mfaSetup.secret} />
              <Anchor href={mfaSetup.qr_code}>Open authenticator enrollment link</Anchor>
              <AppTextInput
                label={t.pages.security.verificationCode}
                placeholder="123456"
                value={mfaCode}
                onChange={(event) => setMfaCode(event.currentTarget.value)}
              />
              <Group>
                <AppButton loading={saving} onClick={enableMfa}>
                  {t.pages.security.verifyAndEnable}
                </AppButton>
                <AppButton appVariant="secondary" onClick={() => setMfaSetup(undefined)}>
                  {t.common.cancel}
                </AppButton>
              </Group>
            </>
          ) : null}
          {user?.mfaEnabled ? (
            <>
              {useBackupCode ? (
                <AppTextInput
                  label={t.pages.security.backupCode}
                  value={mfaCode}
                  onChange={(event) => setMfaCode(event.currentTarget.value)}
                />
              ) : (
                <AppTextInput
                  label={t.forms.authenticatorCode}
                  placeholder="123456"
                  value={mfaCode}
                  onChange={(event) => setMfaCode(event.currentTarget.value)}
                />
              )}
              <Anchor
                size="sm"
                component="button"
                type="button"
                onClick={() => {
                  setUseBackupCode((prev) => !prev);
                  setMfaCode("");
                }}
              >
                {useBackupCode
                  ? t.forms.authenticatorCode
                  : t.pages.security.useBackupCode}
              </Anchor>
              <AppButton appVariant="danger" loading={saving} onClick={disableMfa}>
                {t.pages.security.disableMfa}
              </AppButton>
            </>
          ) : null}
          {backupCodes.length > 0 ? (
            <CodeBlock language="Backup codes" value={backupCodes.join("\n")} />
          ) : null}
        </Stack>
      </SurfaceCard>

      {oauthProviders.length > 0 ? (
        <SurfaceCard title={t.pages.security.connectedAccounts} description={t.pages.security.connectedAccountsDesc}>
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
                    onClick={() => setProviderToUnlink(account.provider)}
                  >
                    {t.pages.security.disconnect}
                  </AppButton>
                </Group>
              ))
            ) : (
              <Text size="sm" color="dimmed">
                {t.pages.security.noAccountsConnected}
              </Text>
            )}
            {unlinkedProviders.length > 0 ? (
              <>
                <Divider />
                <Group>
                  {unlinkedProviders.map((provider) => (
                    <AppButton
                      key={provider.key}
                      appVariant="secondary"
                      onClick={() => linkProvider(provider.key)}
                    >
                      {t.pages.security.connect(provider.label)}
                    </AppButton>
                  ))}
                </Group>
              </>
            ) : null}
          </Stack>
        </SurfaceCard>
      ) : null}

      <SurfaceCard title={t.pages.security.activeSessions}>
        <DataTable
          columns={sessionColumns}
          data={sessions}
          getRowId={(row) => row.id}
          initialPageSize={10}
        />
      </SurfaceCard>

      <SurfaceCard title={t.pages.security.loginHistory}>
        <DataTable
          columns={historyColumns}
          data={history}
          getRowId={(row) => row.id}
          initialPageSize={10}
        />
      </SurfaceCard>

      <ConfirmDialog
        opened={Boolean(sessionToRevoke)}
        title={t.pages.security.revokeSession}
        tone="danger"
        confirmLabel={t.pages.security.revoke}
        loading={saving}
        message={t.pages.security.revokeDesc}
        onCancel={() => setSessionToRevoke(undefined)}
        onConfirm={revokeSession}
      />

      <ConfirmDialog
        opened={Boolean(providerToUnlink)}
        title={t.pages.security.disconnectAccount}
        tone="danger"
        confirmLabel={t.pages.security.disconnect}
        loading={saving}
        message={t.pages.security.disconnectConfirm(providerToUnlink ?? "")}
        onCancel={() => setProviderToUnlink(undefined)}
        onConfirm={unlinkProvider}
      />
    </Stack>
  );
}
