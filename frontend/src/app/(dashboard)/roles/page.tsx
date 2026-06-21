"use client";

import { Stack, Text } from "@mantine/core";
import {
  TableSkeleton,
  AppButton,
  ChipList,
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

type RoleRecord = {
  created_at: string;
  description: string;
  id: string;
  is_system: boolean;
  name: string;
  parent_role_id?: string | null;
  permissions?: Array<{ action: string; id: string; resource: string }>;
  updated_at: string;
};

export default function RolesPage() {
  const t = useTranslations();
  const { create, edit, show } = useNavigation();
  const {
    query,
    result: { data = [] },
  } = useList<RoleRecord>({
    resource: "roles",
    pagination: { mode: "off" },
  });

  const columns = useMemo<ColumnDef<RoleRecord>[]>(
    () => [
      {
        accessorKey: "name",
        header: t.table.columns.role,
        cell: ({ row }) => (
          <div>
            <Text size="sm" weight={600}>
              {row.original.name}
            </Text>
            <Text size="xs" color="dimmed">
              {row.original.description}
            </Text>
          </div>
        ),
      },
      {
        accessorKey: "is_system",
        header: t.table.columns.type,
        cell: ({ row }) => <StatusBadge value={row.original.is_system ? "system" : "custom"} />,
      },
      {
        id: "permissions",
        header: t.table.columns.permissions,
        accessorFn: (row) =>
          row.permissions?.map((permission) => `${permission.resource}:${permission.action}`).join(" ") ?? "",
        cell: ({ row }) => (
          <ChipList
            items={
              row.original.permissions?.map(
                (permission) => `${permission.resource}:${permission.action}`,
              ) ?? []
            }
          />
        ),
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
  if (query.isError) return <ErrorState title={t.pages.roles.couldNotLoad} />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.roles.title}
        description={t.pages.roles.description}
        status={<StatusBadge value={t.common.records(data.length)} />}
        actions={<AppButton onClick={() => create("roles")}>{t.pages.roles.createTitle}</AppButton>}
      />
      <DataTable
        columns={columns}
        data={data}
        getRowId={(row) => row.id}
        renderRowActions={(row) => (
          <RowActionsMenu
            onView={() => show("roles", row.id)}
            onEdit={() => edit("roles", row.id)}
            onDelete={row.is_system ? undefined : () => undefined}
          />
        )}
      />
    </Stack>
  );
}
