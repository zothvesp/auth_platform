"use client";

import { Group, Stack } from "@mantine/core";
import { useRouter } from "next/navigation";
import { useState } from "react";
import {
  AppButton,
  AppTextInput,
  AppTextarea,
  InlineAlert,
  PageHeader,
  SaveButton,
  SurfaceCard,
} from "@components/ui";
import { apiRequest } from "@lib/request";
import { useTranslations } from "@lib/i18n";

export default function CreatePermissionPage() {
  const t = useTranslations();
  const router = useRouter();
  const [resource, setResource] = useState("");
  const [action, setAction] = useState("");
  const [description, setDescription] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string>();

  const submit = async () => {
    setSaving(true);
    setError(undefined);
    try {
      await apiRequest("/permissions", {
        method: "POST",
        body: JSON.stringify({ resource, action, description }),
      });
      router.push("/permissions");
      router.refresh();
    } catch {
      setError("Could not create permission");
    } finally {
      setSaving(false);
    }
  };

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.permissions.createTitle}
        description={t.pages.permissions.createDesc}
      />
      {error ? <InlineAlert tone="error">{error}</InlineAlert> : null}
      <SurfaceCard title={t.pages.permissions.permissionDetails}>
        <Stack p="md" spacing="md">
          <AppTextInput
            label={t.forms.resource}
            description={t.forms.lowerCaseOnly}
            required
            value={resource}
            onChange={(event) => setResource(event.currentTarget.value)}
          />
          <AppTextInput
            label={t.forms.action}
            description={t.forms.lowerCaseOnly}
            required
            value={action}
            onChange={(event) => setAction(event.currentTarget.value)}
          />
          <AppTextarea
            label={t.forms.description}
            required
            value={description}
            onChange={(event) => setDescription(event.currentTarget.value)}
          />
        </Stack>
      </SurfaceCard>
      <Group position="right">
        <AppButton appVariant="secondary" onClick={() => router.push("/permissions")}>
          {t.common.cancel}
        </AppButton>
        <SaveButton loading={saving} onClick={submit}>
          {t.pages.permissions.createPermission}
        </SaveButton>
      </Group>
    </Stack>
  );
}
