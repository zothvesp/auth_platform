"use client";

import { Stack, Text } from "@mantine/core";
import { useRouter } from "next/navigation";
import { AppButton, PageHeader, SurfaceCard } from "@components/ui";
import { useTranslations } from "@lib/i18n";

export default function ForbiddenPage() {
  const t = useTranslations();
  const router = useRouter();

  return (
    <Stack spacing="lg">
      <PageHeader
        title={t.pages.forbidden.title}
        description={t.pages.forbidden.description}
      />
      <SurfaceCard>
        <Stack spacing="md" p="md" align="center">
          <Text size="sm" color="dimmed">
            {t.pages.forbidden.description}
          </Text>
          <AppButton appVariant="secondary" onClick={() => router.push("/")}>
            {t.common.back}
          </AppButton>
        </Stack>
      </SurfaceCard>
    </Stack>
  );
}
