"use client";

import { Group, Stack } from "@mantine/core";
import { useList, useOne } from "@refinedev/core";
import { useRouter } from "next/navigation";
import { useEffect, useMemo, useState } from "react";
import {
  AppButton,
  AppSelect,
  AppTextInput,
  AppTextarea,
  ErrorState,
  FormSkeleton,
  InlineAlert,
  PageHeader,
  PermissionMatrix,
  SaveButton,
  SurfaceCard,
} from "@components/ui";
import type { PermissionRecord, RoleRecord } from "@lib/admin-types";
import { apiRequest } from "@lib/request";
import { useTranslations } from "@lib/i18n";

type RoleEditorProps = {
  id?: string;
};

export function RoleEditor({ id }: RoleEditorProps) {
  const t = useTranslations();
  const router = useRouter();
  const editing = Boolean(id);
  const roleQuery = useOne<RoleRecord>({
    resource: "roles",
    id: id ?? "",
    queryOptions: { enabled: editing },
  });
  const permissionsQuery = useList<PermissionRecord>({
    resource: "permissions",
    pagination: { mode: "off" },
  });
  const rolesQuery = useList<RoleRecord>({
    resource: "roles",
    pagination: { mode: "off" },
  });
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [parentRoleId, setParentRoleId] = useState<string | null>(null);
  const [permissionIds, setPermissionIds] = useState<string[]>([]);
  const [error, setError] = useState<string>();
  const [saving, setSaving] = useState(false);

  const role = roleQuery.result;

  useEffect(() => {
    if (!role) return;
    setName(role.name);
    setDescription(role.description);
    setParentRoleId(role.parent_role_id ?? null);
    setPermissionIds(role.permissions?.map((permission) => permission.id) ?? []);
  }, [role]);

  const groups = useMemo(() => {
    const grouped = new Map<string, PermissionRecord[]>();
    permissionsQuery.result.data.forEach((permission) => {
      grouped.set(permission.resource, [
        ...(grouped.get(permission.resource) ?? []),
        permission,
      ]);
    });

    return Array.from(grouped.entries()).map(([resource, permissions]) => ({
      title: resource,
      permissions: permissions.map((permission) => ({
        value: permission.id,
        label: `${permission.resource}:${permission.action}`,
        description: permission.description,
      })),
    }));
  }, [permissionsQuery.result.data]);

  if (
    permissionsQuery.query.isLoading ||
    rolesQuery.query.isLoading ||
    (editing && roleQuery.query.isLoading)
  ) {
    return <FormSkeleton />;
  }

  if (
    permissionsQuery.query.isError ||
    rolesQuery.query.isError ||
    (editing && roleQuery.query.isError)
  ) {
    return <ErrorState title={t.pages.roles.couldNotLoadOne} />;
  }

  const submit = async () => {
    setSaving(true);
    setError(undefined);

    try {
      if (editing) {
        await apiRequest(`/roles/${id}`, {
          method: "PUT",
          body: JSON.stringify({
            description,
            permission_ids: permissionIds,
          }),
        });
      } else {
        await apiRequest("/roles", {
          method: "POST",
          body: JSON.stringify({
            name,
            description,
            parent_role_id: parentRoleId || null,
            permission_ids: permissionIds,
          }),
        });
      }
      router.push("/roles");
      router.refresh();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not save role");
    } finally {
      setSaving(false);
    }
  };

  return (
    <Stack spacing="lg">
      <PageHeader
        title={editing ? t.pages.roles.editTitle : t.pages.roles.createTitle}
        description={t.pages.roles.createDesc}
      />
      {error ? <InlineAlert tone="error">{error}</InlineAlert> : null}
      <SurfaceCard title={t.pages.roles.roleDetails}>
        <Stack p="md" spacing="md">
          <AppTextInput
            label={t.forms.name}
            description={t.forms.lowerCaseOnly}
            disabled={editing}
            required
            value={name}
            onChange={(event) => setName(event.currentTarget.value)}
          />
          <AppTextarea
            label={t.forms.description}
            required
            value={description}
            onChange={(event) => setDescription(event.currentTarget.value)}
          />
          {!editing ? (
            <AppSelect
              label={t.forms.parentRole}
              placeholder={t.pages.roles.noParentRole}
              value={parentRoleId}
              onChange={setParentRoleId}
              data={rolesQuery.result.data.map((item) => ({
                value: item.id,
                label: item.name,
              }))}
            />
          ) : null}
        </Stack>
      </SurfaceCard>
      <PermissionMatrix groups={groups} value={permissionIds} onChange={setPermissionIds} />
      <Group position="right">
        <AppButton appVariant="secondary" onClick={() => router.push("/roles")}>
          {t.common.cancel}
        </AppButton>
        <SaveButton loading={saving} onClick={submit}>
          {t.pages.roles.saveRole}
        </SaveButton>
      </Group>
    </Stack>
  );
}
