"use client";

import { Group, Stack, Text, ThemeIcon, Title } from "@mantine/core";
import type { ReactNode } from "react";

type RecordHeaderProps = {
  actions?: ReactNode;
  icon?: ReactNode;
  status?: ReactNode;
  subtitle?: ReactNode;
  title: ReactNode;
};

export const RecordHeader = ({ actions, icon, status, subtitle, title }: RecordHeaderProps) => (
  <Group position="apart" align="flex-start" spacing="lg">
    <Group align="flex-start" spacing="sm" noWrap>
      {icon ? (
        <ThemeIcon size={38} radius="md" variant="light" color="cyan">
          {icon}
        </ThemeIcon>
      ) : null}
      <Stack spacing={3}>
        <Group spacing="xs">
          <Title order={2}>{title}</Title>
          {status}
        </Group>
        {subtitle ? (
          <Text size="sm" color="dimmed">
            {subtitle}
          </Text>
        ) : null}
      </Stack>
    </Group>
    {actions ? <Group spacing="xs">{actions}</Group> : null}
  </Group>
);
