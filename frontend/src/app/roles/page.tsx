"use client";

import { Stack, Text } from "@mantine/core";
import {
  AppLoader,
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
  const { edit, show } = useNavigation();
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
        header: "Role",
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
        header: "Type",
        cell: ({ row }) => <StatusBadge value={row.original.is_system ? "system" : "custom"} />,
      },
      {
        id: "permissions",
        header: "Permissions",
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
        header: "Created",
        cell: ({ row }) => <DateTimeText value={row.original.created_at} />,
      },
    ],
    [],
  );

  if (query.isLoading) return <AppLoader label="Loading roles" />;
  if (query.isError) return <ErrorState title="Could not load roles" />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title="Roles"
        description="Manage RBAC roles and their assigned permissions."
        status={<StatusBadge value={`${data.length} roles`} />}
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
