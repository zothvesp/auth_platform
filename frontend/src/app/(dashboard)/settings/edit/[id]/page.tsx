"use client";

import { Group, Stack } from "@mantine/core";
import { useOne } from "@refinedev/core";
import { useParams, useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import {
  AppButton,
  AppSwitch,
  AppTextInput,
  ErrorState,
  FormSkeleton,
  InlineAlert,
  PageHeader,
  SaveButton,
  StatusBadge,
  SurfaceCard,
} from "@components/ui";
import type { SettingRecord } from "@lib/admin-types";
import { apiRequest } from "@lib/request";
import { useTranslations } from "@lib/i18n";

const isOAuthToggle = (key: string) => /^oauth\.\w+_enabled$/.test(key);

export default function EditSettingPage() {
  const t = useTranslations();
  const params = useParams<{ id: string }>();
  const router = useRouter();
  const key = decodeURIComponent(params?.id ?? "");
  const query = useOne<SettingRecord>({ resource: "settings", id: key });
  const [value, setValue] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string>();
  const setting = query.result;

  useEffect(() => {
    if (setting) setValue(setting.value);
  }, [setting]);

  if (query.query.isLoading) return <FormSkeleton />;
  if (query.query.isError || !setting) return <ErrorState title={t.pages.settings.couldNotLoadOne} />;

  const friendlyLabel = (key: string) => {
    if (key === "oauth.google_enabled") return t.pages.settings.enableGoogle;
    if (key === "oauth.github_enabled") return t.pages.settings.enableGitHub;
    if (key === "oauth.microsoft_enabled") return t.pages.settings.enableMicrosoft;
    return key
      .replace(/^oauth\./, "")
      .replace(/_/g, " ")
      .replace(/\b\w/g, (c) => c.toUpperCase());
  };

  const isToggle = isOAuthToggle(setting.key);
  const isEnabled = value === "true";

  const submit = async () => {
    setSaving(true);
    setError(undefined);
    try {
      await apiRequest(`/config/${encodeURIComponent(setting.key)}`, {
        method: "PUT",
        body: JSON.stringify({ value }),
      });
      router.push("/settings");
      router.refresh();
    } catch {
      setError("Could not update setting");
    } finally {
      setSaving(false);
    }
  };

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.settings.editTitle}
        description={setting.description}
        status={<StatusBadge value={setting.is_public ? "public" : "restricted"} />}
      />
      {error ? <InlineAlert tone="error">{error}</InlineAlert> : null}
      <SurfaceCard title={setting.category}>
        <Stack p="md" spacing="md">
          <AppTextInput label={t.forms.key} value={setting.key} disabled />
          {isToggle ? (
            <AppSwitch
              label={friendlyLabel(setting.key)}
              checked={isEnabled}
              onChange={(event) => setValue(event.currentTarget.checked ? "true" : "false")}
            />
          ) : (
            <AppTextInput
              label={t.forms.value}
              required
              value={value}
              onChange={(event) => setValue(event.currentTarget.value)}
            />
          )}
        </Stack>
      </SurfaceCard>
      <Group position="right">
        <AppButton appVariant="secondary" onClick={() => router.push("/settings")}>
          {t.common.cancel}
        </AppButton>
        <SaveButton loading={saving} onClick={submit}>
          {t.pages.settings.saveSetting}
        </SaveButton>
      </Group>
    </Stack>
  );
}
