"use client";

import { Stack } from "@mantine/core";
import { useDelete, useNavigation, useOne } from "@refinedev/core";
import { IconLockAccess } from "@tabler/icons-react";
import { useParams } from "next/navigation";
import { useState } from "react";
import {
  AppButton,
  ConfirmDialog,
  DateTimeText,
  DetailGrid,
  DetailGridSkeleton,
  ErrorState,
  RecordHeader,
} from "@components/ui";
import type { PermissionRecord } from "@lib/admin-types";
import { useTranslations } from "@lib/i18n";

export default function PermissionDetailPage() {
  const t = useTranslations();
  const params = useParams<{ id: string }>();
  const { list } = useNavigation();
  const query = useOne<PermissionRecord>({
    resource: "permissions",
    id: params?.id ?? "",
  });
  const { mutate: deletePermission, mutation } = useDelete();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const permission = query.result;

  if (query.query.isLoading) return <DetailGridSkeleton columns={2} items={4} />;
  if (query.query.isError || !permission) {
    return <ErrorState title={t.pages.permissions.couldNotLoadOne} />;
  }

  return (
    <Stack spacing="lg">
      <RecordHeader
        icon={<IconLockAccess size={20} />}
        title={`${permission.resource}:${permission.action}`}
        subtitle={permission.description}
        actions={
          <AppButton appVariant="danger" onClick={() => setConfirmDelete(true)}>
            {t.common.delete}
          </AppButton>
        }
      />
      <DetailGrid
        items={[
          { label: t.detail.permissionId, value: permission.id },
          { label: t.detail.resource, value: permission.resource },
          { label: t.detail.action, value: permission.action },
          { label: t.detail.created, value: <DateTimeText value={permission.created_at} /> },
        ]}
      />
      <ConfirmDialog
        opened={confirmDelete}
        title={t.confirm.deletePermission}
        tone="danger"
        confirmLabel={t.confirm.deletePermission}
        loading={mutation.isPending}
        message={t.confirm.deletePermissionDesc}
        onCancel={() => setConfirmDelete(false)}
        onConfirm={() =>
          deletePermission(
            { resource: "permissions", id: permission.id },
            { onSuccess: () => list("permissions") },
          )
        }
      />
    </Stack>
  );
}
