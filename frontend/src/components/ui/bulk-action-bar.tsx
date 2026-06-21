"use client";

import { Group, Paper, Text } from "@mantine/core";
import type { ReactNode } from "react";
import { useTranslations } from "@lib/i18n";
import { AppButton } from "./button";

type BulkActionBarProps = {
  actions?: ReactNode;
  onClear?: () => void;
  selectedCount: number;
};

export const BulkActionBar = ({ actions, onClear, selectedCount }: BulkActionBarProps) => {
  const t = useTranslations();
  if (!selectedCount) return null;

  return (
    <Paper withBorder radius="md" p="xs">
      <Group spacing="xs">
        <Text size="xs" color="dimmed">
          {selectedCount} selected
        </Text>
        {actions}
        {onClear ? (
          <AppButton appVariant="ghost" onClick={onClear}>
            {t.common.clear}
          </AppButton>
        ) : null}
      </Group>
    </Paper>
  );
};
