"use client";

import { Group, Stack, Text } from "@mantine/core";
import {
  AppButton,
  TableSkeleton,
  AppSelect,
  ChipList,
  DataTable,
  DateTimeText,
  ErrorState,
  PageHeader,
  RowActionsMenu,
  Toolbar,
  StatusBadge,
  UserAvatar,
} from "@components/ui";
import { useList, useNavigation } from "@refinedev/core";
import type { ColumnDef } from "@tanstack/react-table";
import { useMemo, useState } from "react";
import type { RoleRecord, UserRecord } from "@lib/admin-types";
import { useTranslations } from "@lib/i18n";

export default function UsersPage() {
  const t = useTranslations();
  const { create, edit, show } = useNavigation();
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [roleId, setRoleId] = useState<string | null>(null);
  const rolesQuery = useList<RoleRecord>({
    resource: "roles",
    pagination: { mode: "off" },
  });
  const {
    query,
    result: { data = [], total = 0 },
  } = useList<UserRecord>({
    resource: "users",
    pagination: { currentPage: 1, pageSize: 100 },
    filters: [
      { field: "q", operator: "eq", value: search },
      { field: "status", operator: "eq", value: status },
      { field: "role_id", operator: "eq", value: roleId },
    ],
  });

  const columns = useMemo<ColumnDef<UserRecord>[]>(
    () => [
      {
        id: "user",
        header: t.table.columns.user,
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
        header: t.table.columns.status,
        cell: ({ row }) => <StatusBadge value={row.original.status} />,
      },
      {
        id: "roles",
        header: t.table.columns.roles,
        accessorFn: (row) => row.roles.map((role) => role.name).join(" "),
        cell: ({ row }) => <ChipList items={row.original.roles.map((role) => role.name)} />,
      },
      {
        accessorKey: "mfaEnabled",
        header: t.table.columns.mfa,
        cell: ({ row }) => <StatusBadge value={row.original.mfaEnabled ? "active" : "inactive"} />,
      },
      {
        accessorKey: "lastLoginAt",
        header: t.table.columns.lastLogin,
        cell: ({ row }) => <DateTimeText value={row.original.lastLoginAt} />,
      },
      {
        accessorKey: "createdAt",
        header: t.table.columns.created,
        cell: ({ row }) => <DateTimeText value={row.original.createdAt} />,
      },
    ],
    [t],
  );

  if (query.isLoading) return <TableSkeleton />;
  if (query.isError) return <ErrorState title={t.pages.users.couldNotLoad} />;

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.users.title}
        description={t.pages.users.description}
        status={<StatusBadge value={t.common.records(total)} />}
        actions={<AppButton onClick={() => create("users")}>{t.pages.users.createUser}</AppButton>}
      />
      <Toolbar
        searchValue={search}
        onSearchChange={setSearch}
        searchPlaceholder={t.pages.users.searchPlaceholder}
      >
        <AppSelect
          placeholder={t.pages.users.allStatuses}
          value={status}
          onChange={setStatus}
          data={[
            { value: "active", label: t.common.active },
            { value: "inactive", label: t.common.inactive },
            { value: "suspended", label: t.common.suspended },
          ]}
          w={150}
        />
        <AppSelect
          placeholder={t.pages.users.allRoles}
          value={roleId}
          onChange={setRoleId}
          data={rolesQuery.result.data.map((role) => ({
            value: role.id,
            label: role.name,
          }))}
          w={170}
        />
      </Toolbar>
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
