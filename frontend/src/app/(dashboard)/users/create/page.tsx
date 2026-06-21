"use client";

import { Group, Stack, Text } from "@mantine/core";
import { useList } from "@refinedev/core";
import { useRouter } from "next/navigation";
import { useState } from "react";
import {
  AppButton,
  AppCheckbox,
  AppPasswordInput,
  AppSelect,
  AppSwitch,
  AppTextInput,
  FormSkeleton,
  InlineAlert,
  PageHeader,
  PasswordStrengthMeter,
  SaveButton,
  SurfaceCard,
} from "@components/ui";
import type { RoleRecord, UserRecord } from "@lib/admin-types";
import { apiRequest } from "@lib/request";
import { useTranslations } from "@lib/i18n";

export default function CreateUserPage() {
  const t = useTranslations();
  const router = useRouter();
  const rolesQuery = useList<RoleRecord>({
    resource: "roles",
    pagination: { mode: "off" },
  });
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [status, setStatus] = useState<string | null>("active");
  const [emailVerified, setEmailVerified] = useState(false);
  const [roleIds, setRoleIds] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string>();

  const toggleRole = (roleId: string) => {
    setRoleIds((current) =>
      current.includes(roleId)
        ? current.filter((currentRoleId) => currentRoleId !== roleId)
        : [...current, roleId],
    );
  };

  if (rolesQuery.query.isLoading) return <FormSkeleton />;

  const submit = async () => {
    setSaving(true);
    setError(undefined);
    try {
      const user = await apiRequest<UserRecord>("/admin/users", {
        method: "POST",
        body: JSON.stringify({
          displayName,
          email,
          password,
          status,
          emailVerified,
          roleIds,
        }),
      });
      router.push(`/users/show/${user.id}`);
      router.refresh();
    } catch (cause) {
      setError(
        typeof cause === "object" && cause && "message" in cause
          ? String(cause.message)
          : t.pages.users.couldNotLoad,
      );
    } finally {
      setSaving(false);
    }
  };

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.users.createTitle}
        description={t.pages.users.createDesc}
      />
      {error ? <InlineAlert tone="error">{error}</InlineAlert> : null}
      <SurfaceCard title={t.pages.users.identity}>
        <Stack p="md" spacing="md">
          <AppTextInput
            label={t.forms.displayName}
            required
            value={displayName}
            onChange={(event) => setDisplayName(event.currentTarget.value)}
          />
          <AppTextInput
            label={t.forms.email}
            type="email"
            required
            value={email}
            onChange={(event) => setEmail(event.currentTarget.value)}
          />
          <AppPasswordInput
            label={t.forms.temporaryPassword}
            required
            value={password}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          <PasswordStrengthMeter value={password} />
          <AppSelect
            label={t.forms.initialStatus}
            clearable={false}
            value={status}
            onChange={setStatus}
            data={[
              { value: "active", label: t.common.active },
              { value: "inactive", label: t.common.inactive },
              { value: "suspended", label: t.common.suspended },
            ]}
          />
          <AppSwitch
            label={t.forms.markEmailVerified}
            checked={emailVerified}
            onChange={(event) => setEmailVerified(event.currentTarget.checked)}
          />
        </Stack>
      </SurfaceCard>
      <SurfaceCard
        title={t.pages.users.role}
        description={t.pages.users.rolesHint}
      >
        <Stack p="md" spacing="sm">
          {rolesQuery.result.data.map((role) => (
            <AppCheckbox
              key={role.id}
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
        <AppButton appVariant="secondary" onClick={() => router.push("/users")}>
          {t.common.cancel}
        </AppButton>
        <SaveButton loading={saving} onClick={submit}>
          {t.pages.users.createUser}
        </SaveButton>
      </Group>
    </Stack>
  );
}
