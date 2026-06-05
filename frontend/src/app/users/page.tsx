"use client";

import { Group, Stack, Text } from "@mantine/core";
import {
  AppLoader,
  ChipList,
  DataTable,
  DateTimeText,
  ErrorState,
  PageHeader,
  RowActionsMenu,
  StatusBadge,
  UserAvatar,
} from "@components/ui";
import { useList, useNavigation } from "@refinedev/core";
import type { ColumnDef } from "@tanstack/react-table";
import { useMemo } from "react";

type UserRecord = {
  authMethod: string;
  avatarUrl?: string | null;
  createdAt: string;
  displayName: string;
  email: string;
  emailVerified: boolean;
  id: string;
  lastLoginAt?: string | null;
  mfaEnabled: boolean;
  permissions: string[];
  roles: Array<{ description: string; id: string; name: string }>;
  status: string;
};

export default function UsersPage() {
  const { edit, show } = useNavigation();
  const {
    query,
    result: { data = [] },
  } = useList<UserRecord>({
    resource: "users",
    pagination: { currentPage: 1, pageSize: 100 },
  });

  const columns = useMemo<ColumnDef<UserRecord>[]>(
    () => [
      {
        id: "user",
        header: "User",
        accessorFn: (row) => `${row.displayName} ${row.email}`,
        cell: ({ row }) => (
          <Group spacing="sm" noWrap>
            <UserAvatar
              name={row.original.displayName}
              email={row.original.email}
              statusColor={row.original.status === "active" ? "green" : "yellow"}
              size="sm"
            />
            <div>
              <Text size="sm" weight={600}>
                {row.original.displayName}
              </Text>
              <Text size="xs" color="dimmed">
                {row.original.email}
              </Text>
            </div>
          </Group>
        ),
      },
      {
        accessorKey: "status",
        header: "Status",
        cell: ({ row }) => <StatusBadge value={row.original.status} />,
      },
      {
        id: "roles",
        header: "Roles",
        accessorFn: (row) => row.roles.map((role) => role.name).join(" "),
        cell: ({ row }) => <ChipList items={row.original.roles.map((role) => role.name)} />,
      },
      {
        accessorKey: "mfaEnabled",
        header: "MFA",
        cell: ({ row }) => <StatusBadge value={row.original.mfaEnabled ? "active" : "inactive"} />,
      },
      {
        accessorKey: "lastLoginAt",
        header: "Last login",
        cell: ({ row }) => <DateTimeText value={row.original.lastLoginAt} />,
      },
      {
        accessorKey: "createdAt",
        header: "Created",
        cell: ({ row }) => <DateTimeText value={row.original.createdAt} />,
      },
    ],
    [],
  );

  if (query.isLoading) return <AppLoader label="Loading users" />;
  if (query.isError) return <ErrorState title="Could not load users" />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title="Users"
        description="Manage identities, account status, MFA posture, and role assignments."
        status={<StatusBadge value={`${data.length} records`} />}
      />
      <DataTable
        columns={columns}
        data={data}
        enableRowSelection
        getRowId={(row) => row.id}
        renderRowActions={(row) => (
          <RowActionsMenu
            onView={() => show("users", row.id)}
            onEdit={() => edit("users", row.id)}
            onArchive={() => undefined}
          />
        )}
      />
    </Stack>
  );
}
