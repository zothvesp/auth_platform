"use client";

import { Stack, Text } from "@mantine/core";
import {
  TableSkeleton,
  CodeBlock,
  DataTable,
  DateTimeText,
  ErrorState,
  PageHeader,
  StatusBadge,
} from "@components/ui";
import { useList } from "@refinedev/core";
import type { ColumnDef } from "@tanstack/react-table";
import { useMemo } from "react";
import { useTranslations } from "@lib/i18n";

type AuditLogRecord = {
  action: string;
  created_at: string;
  details?: unknown;
  id: string;
  ip_address: string;
  resource: string;
  resource_id?: string | null;
  success: boolean;
  user_email?: string | null;
  user_id?: string | null;
};

export default function AuditLogsPage() {
  const t = useTranslations();
  const {
    query,
    result: { data = [] },
  } = useList<AuditLogRecord>({
    resource: "audit_logs",
    pagination: { currentPage: 1, pageSize: 200 },
  });

  const columns = useMemo<ColumnDef<AuditLogRecord>[]>(
    () => [
      {
        accessorKey: "created_at",
        header: t.table.columns.time,
        cell: ({ row }) => <DateTimeText value={row.original.created_at} />,
      },
      {
        accessorKey: "action",
        header: t.table.columns.action,
        cell: ({ row }) => <StatusBadge value={row.original.action} />,
      },
      {
        accessorKey: "resource",
        header: t.table.columns.resource,
      },
      {
        accessorKey: "user_email",
        header: t.table.columns.actor,
        cell: ({ row }) => (
          <Text size="sm">{row.original.user_email ?? row.original.user_id ?? "System"}</Text>
        ),
      },
      {
        accessorKey: "ip_address",
        header: t.table.columns.ip,
      },
      {
        accessorKey: "success",
        header: t.table.columns.result,
        cell: ({ row }) => <StatusBadge value={row.original.success ? "success" : "failed"} />,
      },
      {
        id: "details",
        header: t.table.columns.details,
        enableSorting: false,
        cell: ({ row }) =>
          row.original.details ? (
            <CodeBlock copyable={false} value={JSON.stringify(row.original.details)} />
          ) : (
            "-"
          ),
      },
    ],
    [t],
  );

  if (query.isLoading) return <TableSkeleton />;
  if (query.isError) return <ErrorState title={t.pages.auditLogs.couldNotLoad} />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.auditLogs.title}
        description={t.pages.auditLogs.description}
        status={<StatusBadge value={t.common.records(data.length)} />}
      />
      <DataTable columns={columns} data={data} getRowId={(row) => row.id} initialPageSize={20} />
    </Stack>
  );
}
