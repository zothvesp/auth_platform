"use client";

import { CopyButton, Group, Paper, Stack, Text } from "@mantine/core";
import { useNavigation, useOne } from "@refinedev/core";
import { useParams, useSearchParams } from "next/navigation";
import {
  AppButton,
  DetailGrid,
  DetailGridSkeleton,
  ErrorState,
  InlineAlert,
  RecordHeader,
  StatusBadge,
} from "@components/ui";
import type { OAuthAppRecord } from "@lib/admin-types";
import { useTranslations } from "@lib/i18n";

export default function OAuthAppDetailPage() {
  const t = useTranslations();
  const params = useParams<{ id: string }>();
  const searchParams = useSearchParams();
  const secretFromCreate = searchParams?.get("client_secret");
  const { edit } = useNavigation();
  const query = useOne<OAuthAppRecord>({ resource: "oauth_apps", id: params?.id ?? "" });
  const app = query.result;

  if (query.query.isLoading) return <DetailGridSkeleton columns={3} />;
  if (query.query.isError || !app)
    return <ErrorState title={t.pages.oauthApps.couldNotLoadOne} />;

  return (
    <Stack spacing="lg">
      <RecordHeader
        title={app.name}
        subtitle={app.clientId}
        status={<StatusBadge value={app.isActive ? "active" : "inactive"} />}
        actions={
          <AppButton appVariant="secondary" onClick={() => edit("oauth_apps", app.id)}>
            {t.common.edit}
          </AppButton>
        }
      />
      <DetailGrid
        columns={3}
        items={[
          { label: t.pages.oauthApps.clientId, value: app.clientId },
          { label: t.forms.description, value: app.description || t.common.none },
          {
            label: t.forms.pkceRequired,
            value: app.pkceRequired ? t.common.enabled : t.common.disabled,
          },
        ]}
      />
      {secretFromCreate ? (
        <Paper withBorder p="md" radius="md">
          <Stack spacing="sm">
            <Group position="apart">
              <Text size="sm" weight={600}>
                {t.pages.oauthApps.clientSecret}
              </Text>
              <CopyButton value={secretFromCreate}>
                {({ copied, copy }) => (
                  <AppButton appVariant="secondary" onClick={copy}>
                    {copied ? "Copied" : t.common.copy}
                  </AppButton>
                )}
              </CopyButton>
            </Group>
            <Text size="xs" color="dimmed">
              {t.pages.oauthApps.clientSecretShown}
            </Text>
            <Paper p="xs" radius="sm" bg="dark.6">
              <Text size="xs" ff="monospace" selectAll>
                {secretFromCreate}
              </Text>
            </Paper>
          </Stack>
        </Paper>
      ) : null}
    </Stack>
  );
}
