"use client";

import { Group, Modal, Stack, Text } from "@mantine/core";
import { IconAlertTriangle } from "@tabler/icons-react";
import type { ReactNode } from "react";
import { useTranslations } from "@lib/i18n";
import { AppButton } from "./button";

type ConfirmDialogTone = "danger" | "warning" | "default";

type ConfirmDialogProps = {
  cancelLabel?: string;
  children?: ReactNode;
  confirmLabel?: string;
  loading?: boolean;
  message?: ReactNode;
  onCancel: () => void;
  onConfirm: () => void;
  opened: boolean;
  title: string;
  tone?: ConfirmDialogTone;
};

const toneColor: Record<ConfirmDialogTone, string> = {
  danger: "red",
  warning: "yellow",
  default: "cyan",
};

export const ConfirmDialog = ({
  cancelLabel,
  children,
  confirmLabel,
  loading,
  message,
  onCancel,
  onConfirm,
  opened,
  title,
  tone = "default",
}: ConfirmDialogProps) => {
  const t = useTranslations();
  return (
    <Modal opened={opened} onClose={onCancel} title={title} centered radius="md" size="sm">
      <Stack spacing="md">
        <Group align="flex-start" noWrap>
          <IconAlertTriangle size={22} color={`var(--mantine-color-${toneColor[tone]}-5)`} />
          <Text size="sm" color="dimmed">
            {message ?? children}
          </Text>
        </Group>
        <Group position="right" spacing="xs">
          <AppButton appVariant="secondary" onClick={onCancel}>
            {cancelLabel ?? t.common.cancel}
          </AppButton>
          <AppButton
            appVariant={tone === "danger" ? "danger" : "primary"}
            loading={loading}
            onClick={onConfirm}
          >
            {confirmLabel ?? t.common.confirm}
          </AppButton>
        </Group>
      </Stack>
    </Modal>
  );
};
