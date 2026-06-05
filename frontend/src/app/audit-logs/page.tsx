"use client";

import { Stack, Text } from "@mantine/core";
import {
  AppLoader,
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
        header: "Time",
        cell: ({ row }) => <DateTimeText value={row.original.created_at} />,
      },
      {
        accessorKey: "action",
        header: "Action",
        cell: ({ row }) => <StatusBadge value={row.original.action} />,
      },
      {
        accessorKey: "resource",
        header: "Resource",
      },
      {
        accessorKey: "user_email",
        header: "Actor",
        cell: ({ row }) => (
          <Text size="sm">{row.original.user_email ?? row.original.user_id ?? "System"}</Text>
        ),
      },
      {
        accessorKey: "ip_address",
        header: "IP",
      },
      {
        accessorKey: "success",
        header: "Result",
        cell: ({ row }) => <StatusBadge value={row.original.success ? "success" : "failed"} />,
      },
      {
        id: "details",
        header: "Details",
        enableSorting: false,
        cell: ({ row }) =>
          row.original.details ? (
            <CodeBlock copyable={false} value={JSON.stringify(row.original.details)} />
          ) : (
            "-"
          ),
      },
    ],
    [],
  );

  if (query.isLoading) return <AppLoader label="Loading audit logs" />;
  if (query.isError) return <ErrorState title="Could not load audit logs" />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title="Audit logs"
        description="Inspect authentication, RBAC, and administrative events."
        status={<StatusBadge value={`${data.length} events`} />}
      />
      <DataTable columns={columns} data={data} getRowId={(row) => row.id} initialPageSize={20} />
    </Stack>
  );
}
