"use client";

import { Stack, Text } from "@mantine/core";
import {
  AppButton,
  DataTable,
  ErrorState,
  PageHeader,
  RowActionsMenu,
  StatusBadge,
  TableSkeleton,
} from "@components/ui";
import { useList, useNavigation } from "@refinedev/core";
import type { ColumnDef } from "@tanstack/react-table";
import { useMemo } from "react";
import type { OAuthAppRecord } from "@lib/admin-types";
import { useTranslations } from "@lib/i18n";

export default function OAuthAppsPage() {
  const t = useTranslations();
  const { create, edit, show } = useNavigation();
  const {
    query,
    result: { data = [], total = 0 },
  } = useList<OAuthAppRecord>({
    resource: "oauth_apps",
    pagination: { currentPage: 1, pageSize: 100 },
  });

  const columns = useMemo<ColumnDef<OAuthAppRecord>[]>(
    () => [
      {
        accessorKey: "name",
        header: t.table.columns.name,
        cell: ({ row }) => (
          <Text size="sm" weight={600}>
            {row.original.name}
          </Text>
        ),
      },
      {
        accessorKey: "clientId",
        header: t.table.columns.clientId,
        cell: ({ row }) => (
          <Text size="xs" color="dimmed" ff="monospace">
            {row.original.clientId}
          </Text>
        ),
      },
      {
        accessorKey: "isActive",
        header: t.table.columns.active,
        cell: ({ row }) => (
          <StatusBadge value={row.original.isActive ? "active" : "inactive"} />
        ),
      },
    ],
    [t],
  );

  if (query.isLoading) return <TableSkeleton />;
  if (query.isError) return <ErrorState title={t.pages.oauthApps.couldNotLoad} />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.oauthApps.title}
        description={t.pages.oauthApps.description}
        status={<StatusBadge value={t.common.records(total)} />}
        actions={
          <AppButton onClick={() => create("oauth_apps")}>
            {t.pages.oauthApps.createApp}
          </AppButton>
        }
      />
      <DataTable
        columns={columns}
        data={data}
        getRowId={(row) => row.id}
        renderRowActions={(row) => (
          <RowActionsMenu
            onView={() => show("oauth_apps", row.id)}
            onEdit={() => edit("oauth_apps", row.id)}
            onArchive={() => undefined}
          />
        )}
      />
    </Stack>
  );
}
