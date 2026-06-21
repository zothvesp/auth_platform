"use client";

import { Stack, Text } from "@mantine/core";
import {
  TableSkeleton,
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
  const t = useTranslations();
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
        header: t.table.columns.key,
        cell: ({ row }) => (
          <Text size="sm" weight={700}>
            {row.original.key}
          </Text>
        ),
      },
      {
        accessorKey: "value",
        header: t.table.columns.value,
      },
      {
        accessorKey: "category",
        header: t.table.columns.category,
      },
      {
        accessorKey: "is_public",
        header: t.table.columns.visibility,
        cell: ({ row }) => <StatusBadge value={row.original.is_public ? "public" : "restricted"} />,
      },
      {
        accessorKey: "description",
        header: t.table.columns.description,
      },
      {
        accessorKey: "updated_at",
        header: t.table.columns.updated,
        cell: ({ row }) => <DateTimeText value={row.original.updated_at} />,
      },
    ],
    [t],
  );

  if (query.isLoading) return <TableSkeleton />;
  if (query.isError) return <ErrorState title={t.pages.settings.couldNotLoad} />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.settings.title}
        description={t.pages.settings.description}
        status={<StatusBadge value={t.common.records(data.length)} />}
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
