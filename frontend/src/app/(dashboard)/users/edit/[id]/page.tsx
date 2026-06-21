"use client";

import { Checkbox, Group, Stack, Text } from "@mantine/core";
import { useList, useOne } from "@refinedev/core";
import { useParams, useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import {
  AppButton,
  AppTextInput,
  ErrorState,
  FormSkeleton,
  InlineAlert,
  PageHeader,
  SaveButton,
  SurfaceCard,
} from "@components/ui";
import type { RoleRecord, UserRecord } from "@lib/admin-types";
import { apiRequest } from "@lib/request";
import { useTranslations } from "@lib/i18n";

export default function EditUserPage() {
  const t = useTranslations();
  const params = useParams<{ id: string }>();
  const router = useRouter();
  const userQuery = useOne<UserRecord>({
    resource: "users",
    id: params?.id ?? "",
  });
  const rolesQuery = useList<RoleRecord>({
    resource: "roles",
    pagination: { mode: "off" },
  });
  const [displayName, setDisplayName] = useState("");
  const [avatarUrl, setAvatarUrl] = useState("");
  const [roleIds, setRoleIds] = useState<string[]>([]);
  const [originalRoleIds, setOriginalRoleIds] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string>();
  const user = userQuery.result;

  useEffect(() => {
    if (!user) return;
    const assigned = user.roles.map((role) => role.id);
    setDisplayName(user.displayName);
    setAvatarUrl(user.avatarUrl ?? "");
    setRoleIds(assigned);
    setOriginalRoleIds(assigned);
  }, [user]);

  if (userQuery.query.isLoading || rolesQuery.query.isLoading) {
    return <FormSkeleton />;
  }
  if (userQuery.query.isError || rolesQuery.query.isError || !user) {
    return <ErrorState title={t.pages.users.couldNotLoadEditor} />;
  }

  const toggleRole = (id: string) => {
    setRoleIds((current) =>
      current.includes(id) ? current.filter((roleId) => roleId !== id) : [...current, id],
    );
  };

  const submit = async () => {
    setSaving(true);
    setError(undefined);
    try {
      await apiRequest(`/admin/users/${user.id}`, {
        method: "PUT",
        body: JSON.stringify({ displayName, avatarUrl: avatarUrl || null }),
      });

      const added = roleIds.filter((id) => !originalRoleIds.includes(id));
      const removed = originalRoleIds.filter((id) => !roleIds.includes(id));

      await Promise.all([
        ...added.map((roleId) =>
          apiRequest(`/admin/users/${user.id}/roles`, {
            method: "POST",
            body: JSON.stringify({ roleId }),
          }),
        ),
        ...removed.map((roleId) =>
          apiRequest(`/admin/users/${user.id}/roles`, {
            method: "DELETE",
            body: JSON.stringify({ roleId }),
          }),
        ),
      ]);

      router.push(`/users/show/${user.id}`);
      router.refresh();
    } catch {
      setError("Could not save user changes");
    } finally {
      setSaving(false);
    }
  };

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.users.editTitle}
        description={`Update profile and role assignments for ${user.email}.`}
      />
      {error ? <InlineAlert tone="error">{error}</InlineAlert> : null}
      <SurfaceCard title={t.pages.users.profile}>
        <Stack p="md" spacing="md">
          <AppTextInput label={t.forms.email} value={user.email} disabled />
          <AppTextInput
            label={t.forms.displayName}
            required
            value={displayName}
            onChange={(event) => setDisplayName(event.currentTarget.value)}
          />
          <AppTextInput
            label={t.forms.avatarUrl}
            value={avatarUrl}
            onChange={(event) => setAvatarUrl(event.currentTarget.value)}
          />
        </Stack>
      </SurfaceCard>
      <SurfaceCard title={t.pages.users.assignedRoles} description={t.pages.users.rolesDesc}>
        <Stack p="md" spacing="sm">
          {rolesQuery.result.data.map((role) => (
            <Checkbox
              key={role.id}
              color="cyan"
              checked={roleIds.includes(role.id)}
              label={
                <div>
                  <Text size="sm" weight={600}>
                    {role.name}
                  </Text>
                  <Text size="xs" color="dimmed">
                    {role.description}
                  </Text>
                </div>
              }
              onChange={() => toggleRole(role.id)}
            />
          ))}
        </Stack>
      </SurfaceCard>
      <Group position="right">
        <AppButton
          appVariant="secondary"
          onClick={() => router.push(`/users/show/${user.id}`)}
        >
          {t.common.cancel}
        </AppButton>
        <SaveButton loading={saving} onClick={submit}>
          {t.pages.users.saveUser}
        </SaveButton>
      </Group>
    </Stack>
  );
}
