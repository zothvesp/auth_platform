"use client";

import { Group, Stack } from "@mantine/core";
import { useDelete, useNavigation, useOne } from "@refinedev/core";
import { IconShield } from "@tabler/icons-react";
import { useParams } from "next/navigation";
import {
  AppButton,
  ChipList,
  ConfirmDialog,
  DateTimeText,
  DetailGrid,
  DetailGridSkeleton,
  ErrorState,
  RecordHeader,
  StatusBadge,
  SurfaceCard,
} from "@components/ui";
import type { RoleRecord } from "@lib/admin-types";
import { useState } from "react";
import { useTranslations } from "@lib/i18n";

export default function RoleDetailPage() {
  const t = useTranslations();
  const params = useParams<{ id: string }>();
  const { edit, list } = useNavigation();
  const { mutate: deleteRole, mutation } = useDelete();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const query = useOne<RoleRecord>({ resource: "roles", id: params?.id ?? "" });
  const role = query.result;

  if (query.query.isLoading) return <DetailGridSkeleton columns={2} items={4} />;
  if (query.query.isError || !role) return <ErrorState title={t.pages.roles.couldNotLoadOne} />;

  return (
    <Stack spacing="lg">
      <RecordHeader
        icon={<IconShield size={20} />}
        title={role.name}
        subtitle={role.description}
        status={<StatusBadge value={role.is_system ? "system" : "custom"} />}
        actions={
          <>
            <AppButton appVariant="secondary" onClick={() => edit("roles", role.id)}>
              {t.common.edit}
            </AppButton>
            {!role.is_system ? (
              <AppButton appVariant="danger" onClick={() => setConfirmDelete(true)}>
                {t.common.delete}
              </AppButton>
            ) : null}
          </>
        }
      />
      <DetailGrid
        items={[
          { label: t.detail.roleId, value: role.id },
          { label: t.detail.parentRole, value: role.parent_role_id ?? t.pages.roles.noParentRole },
          { label: t.detail.created, value: <DateTimeText value={role.created_at} /> },
          { label: t.detail.updated, value: <DateTimeText value={role.updated_at} /> },
        ]}
      />
      <SurfaceCard title={t.pages.roles.grantedPermissions}>
        <Group p="md">
          <ChipList
            items={
              role.permissions?.map(
                (permission) => `${permission.resource}:${permission.action}`,
              ) ?? []
            }
          />
        </Group>
      </SurfaceCard>
      <ConfirmDialog
        opened={confirmDelete}
        title={t.confirm.deleteRole}
        tone="danger"
        confirmLabel={t.confirm.deleteRole}
        loading={mutation.isPending}
        message={t.confirm.deleteRoleDesc(role.name)}
        onCancel={() => setConfirmDelete(false)}
        onConfirm={() =>
          deleteRole(
            { resource: "roles", id: role.id },
            { onSuccess: () => list("roles") },
          )
        }
      />
    </Stack>
  );
}
