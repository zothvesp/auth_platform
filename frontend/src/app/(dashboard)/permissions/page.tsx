"use client";

import { Stack, Text } from "@mantine/core";
import {
  TableSkeleton,
  AppButton,
  DataTable,
  DateTimeText,
  ErrorState,
  PageHeader,
  RowActionsMenu,
  StatusBadge,
} from "@components/ui";
import { useList, useNavigation } from "@refinedev/core";
import type { ColumnDef } from "@tanstack/react-table";
import { useMemo } from "react";
import { useTranslations } from "@lib/i18n";

type PermissionRecord = {
  action: string;
  created_at: string;
  description: string;
  id: string;
  resource: string;
};

export default function PermissionsPage() {
  const t = useTranslations();
  const { create, show } = useNavigation();
  const {
    query,
    result: { data = [] },
  } = useList<PermissionRecord>({
    resource: "permissions",
    pagination: { mode: "off" },
  });

  const columns = useMemo<ColumnDef<PermissionRecord>[]>(
    () => [
      {
        id: "key",
        header: t.table.columns.permission,
        accessorFn: (row) => `${row.resource}:${row.action}`,
        cell: ({ row }) => (
          <Text size="sm" weight={700}>
            {row.original.resource}:{row.original.action}
          </Text>
        ),
      },
      {
        accessorKey: "resource",
        header: t.table.columns.resource,
      },
      {
        accessorKey: "action",
        header: t.table.columns.action,
      },
      {
        accessorKey: "description",
        header: t.table.columns.description,
      },
      {
        accessorKey: "created_at",
        header: t.detail.created,
        cell: ({ row }) => <DateTimeText value={row.original.created_at} />,
      },
    ],
    [t],
  );

  if (query.isLoading) return <TableSkeleton />;
  if (query.isError) return <ErrorState title={t.pages.permissions.couldNotLoad} />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.permissions.title}
        description={t.pages.permissions.description}
        status={<StatusBadge value={t.common.records(data.length)} />}
        actions={<AppButton onClick={() => create("permissions")}>{t.pages.permissions.createPermission}</AppButton>}
      />
      <DataTable
        columns={columns}
        data={data}
        getRowId={(row) => row.id}
        renderRowActions={(row) => <RowActionsMenu onView={() => show("permissions", row.id)} />}
      />
    </Stack>
  );
}
