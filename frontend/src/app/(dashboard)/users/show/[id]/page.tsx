"use client";

import { Group, Stack } from "@mantine/core";
import { useNavigation, useOne } from "@refinedev/core";
import { IconUser } from "@tabler/icons-react";
import { useParams, useRouter } from "next/navigation";
import { useState } from "react";
import {
  AppButton,
  ChipList,
  ConfirmDialog,
  DateTimeText,
  DetailGrid,
  DetailGridSkeleton,
  ErrorState,
  InlineAlert,
  RecordHeader,
  StatusBadge,
  SurfaceCard,
} from "@components/ui";
import type { UserRecord } from "@lib/admin-types";
import { apiRequest } from "@lib/request";
import { useTranslations } from "@lib/i18n";

export default function UserDetailPage() {
  const t = useTranslations();
  const params = useParams<{ id: string }>();
  const router = useRouter();
  const { edit } = useNavigation();
  const query = useOne<UserRecord>({ resource: "users", id: params?.id ?? "" });
  const [action, setAction] = useState<"status" | "delete">();
  const [error, setError] = useState<string>();
  const user = query.result;

  if (query.query.isLoading) return <DetailGridSkeleton columns={3} />;
  if (query.query.isError || !user) return <ErrorState title={t.pages.users.couldNotLoadOne} />;

  const changeStatus = async () => {
    const endpoint = user.status === "active" ? "deactivate" : "reactivate";
    setError(undefined);
    try {
      await apiRequest(`/admin/users/${user.id}/${endpoint}`, { method: "PATCH" });
      setAction(undefined);
      await query.query.refetch();
    } catch {
      setError("Could not update account status");
    }
  };

  const deleteUser = async () => {
    setError(undefined);
    try {
      await apiRequest(`/admin/users/${user.id}`, { method: "DELETE" });
      router.push("/users");
      router.refresh();
    } catch {
      setError("Could not delete user");
    }
  };

  return (
    <Stack spacing="lg">
      <RecordHeader
        icon={<IconUser size={20} />}
        title={user.displayName}
        subtitle={user.email}
        status={<StatusBadge value={user.status} />}
        actions={
          <>
            <AppButton appVariant="secondary" onClick={() => edit("users", user.id)}>
              {t.common.edit}
            </AppButton>
            <AppButton appVariant="secondary" onClick={() => setAction("status")}>
              {user.status === "active" ? t.pages.users.deactivate : t.pages.users.reactivate}
            </AppButton>
            <AppButton appVariant="danger" onClick={() => setAction("delete")}>
              {t.common.delete}
            </AppButton>
          </>
        }
      />
      {error ? <InlineAlert tone="error">{error}</InlineAlert> : null}
      <DetailGrid
        columns={3}
        items={[
          { label: t.detail.userId, value: user.id },
          { label: t.detail.emailVerified, value: user.emailVerified ? t.common.yes : t.common.no },
          { label: t.detail.mfa, value: user.mfaEnabled ? t.detail.enabled : t.detail.disabled },
          { label: t.detail.authentication, value: user.authMethod },
          {
            label: t.detail.lastLogin,
            value: user.lastLoginAt ? <DateTimeText value={user.lastLoginAt} /> : t.common.never,
          },
          { label: t.detail.created, value: <DateTimeText value={user.createdAt} /> },
        ]}
      />
      <SurfaceCard title={t.pages.users.assignedRoles}>
        <Group p="md">
          <ChipList items={user.roles.map((role) => role.name)} />
        </Group>
      </SurfaceCard>
      <SurfaceCard title={t.pages.users.effectivePermissions}>
        <Group p="md">
          <ChipList items={user.permissions} />
        </Group>
      </SurfaceCard>
      <ConfirmDialog
        opened={action === "status"}
        title={user.status === "active" ? t.confirm.deactivateAccount : t.confirm.reactivateAccount}
        tone="warning"
        confirmLabel={user.status === "active" ? t.confirm.deactivate : t.confirm.reactivate}
        message={
          user.status === "active"
            ? t.confirm.deactivateAccountDesc
            : t.confirm.reactivateAccountDesc
        }
        onCancel={() => setAction(undefined)}
        onConfirm={changeStatus}
      />
      <ConfirmDialog
        opened={action === "delete"}
        title={t.confirm.deleteUser}
        tone="danger"
        confirmLabel={t.pages.users.deleteUser}
        message={t.confirm.deleteUserDesc(user.email)}
        onCancel={() => setAction(undefined)}
        onConfirm={deleteUser}
      />
    </Stack>
  );
}
