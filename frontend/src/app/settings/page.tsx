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

type SettingRecord = {
  category: string;
  description: string;
  id?: string;
  is_public: boolean;
  key: string;
  updated_at: string;
  value: string;
};

export default function SettingsPage() {
  const { edit } = useNavigation();
  const {
    query,
    result: { data = [] },
  } = useList<SettingRecord>({
    resource: "settings",
    pagination: { mode: "off" },
  });

  const columns = useMemo<ColumnDef<SettingRecord>[]>(
    () => [
      {
        accessorKey: "key",
        header: "Key",
        cell: ({ row }) => (
          <Text size="sm" weight={700}>
            {row.original.key}
          </Text>
        ),
      },
      {
        accessorKey: "value",
        header: "Value",
      },
      {
        accessorKey: "category",
        header: "Category",
      },
      {
        accessorKey: "is_public",
        header: "Visibility",
        cell: ({ row }) => <StatusBadge value={row.original.is_public ? "public" : "restricted"} />,
      },
      {
        accessorKey: "description",
        header: "Description",
      },
      {
        accessorKey: "updated_at",
        header: "Updated",
        cell: ({ row }) => <DateTimeText value={row.original.updated_at} />,
      },
    ],
    [],
  );

  if (query.isLoading) return <AppLoader label="Loading settings" />;
  if (query.isError) return <ErrorState title="Could not load settings" />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title="Settings"
        description="Review runtime configuration stored in the backend database."
        status={<StatusBadge value={`${data.length} settings`} />}
      />
      <DataTable
        columns={columns}
        data={data}
        getRowId={(row) => row.key}
        renderRowActions={(row) => <RowActionsMenu onEdit={() => edit("settings", row.key)} />}
      />
    </Stack>
  );
}
