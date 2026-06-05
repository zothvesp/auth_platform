"use client";

import { Stack, Text } from "@mantine/core";
import {
  AppLoader,
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

type PermissionRecord = {
  action: string;
  created_at: string;
  description: string;
  id: string;
  resource: string;
};

export default function PermissionsPage() {
  const { show } = useNavigation();
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
        header: "Permission",
        accessorFn: (row) => `${row.resource}:${row.action}`,
        cell: ({ row }) => (
          <Text size="sm" weight={700}>
            {row.original.resource}:{row.original.action}
          </Text>
        ),
      },
      {
        accessorKey: "resource",
        header: "Resource",
      },
      {
        accessorKey: "action",
        header: "Action",
      },
      {
        accessorKey: "description",
        header: "Description",
      },
      {
        accessorKey: "created_at",
        header: "Created",
        cell: ({ row }) => <DateTimeText value={row.original.created_at} />,
      },
    ],
    [],
  );

  if (query.isLoading) return <AppLoader label="Loading permissions" />;
  if (query.isError) return <ErrorState title="Could not load permissions" />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title="Permissions"
        description="Review the system permission keys used by backend authorization checks."
        status={<StatusBadge value={`${data.length} permissions`} />}
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
