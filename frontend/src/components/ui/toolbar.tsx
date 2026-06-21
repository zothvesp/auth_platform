"use client";

import { Group, Paper, TextInput } from "@mantine/core";
import { IconSearch, IconX } from "@tabler/icons-react";
import type { ReactNode } from "react";
import { useTranslations } from "@lib/i18n";
import { AppIconButton } from "./icon-button";

type ToolbarProps = {
  actions?: ReactNode;
  children?: ReactNode;
  onSearchChange?: (value: string) => void;
  searchPlaceholder?: string;
  searchValue?: string;
};

export const Toolbar = ({
  actions,
  children,
  onSearchChange,
  searchPlaceholder,
  searchValue,
}: ToolbarProps) => {
  const t = useTranslations();
  return (
    <Paper withBorder radius="md" p="xs">
      <Group position="apart" align="center">
        <Group spacing="xs">
          {onSearchChange ? (
            <TextInput
              icon={<IconSearch size={15} />}
              placeholder={searchPlaceholder ?? t.common.search}
              value={searchValue ?? ""}
              onChange={(event) => onSearchChange(event.currentTarget.value)}
              rightSection={
                searchValue ? (
                  <AppIconButton
                    label={t.common.clearSearch}
                    icon={<IconX size={14} />}
                    size="sm"
                    onClick={() => onSearchChange("")}
                  />
                ) : null
              }
              size="xs"
              w={260}
            />
          ) : null}
          {children}
        </Group>
        {actions ? <Group spacing="xs">{actions}</Group> : null}
      </Group>
    </Paper>
  );
};
