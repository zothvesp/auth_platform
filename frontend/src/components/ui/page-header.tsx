"use client";

import { Group, Stack, Text, Title } from "@mantine/core";
import type { ReactNode } from "react";

type PageHeaderProps = {
  title: string;
  description?: string;
  status?: ReactNode;
  actions?: ReactNode;
};

export const PageHeader = ({ title, description, status, actions }: PageHeaderProps) => (
  <Group position="apart" align="flex-start" spacing="lg">
    <Stack spacing={3}>
      <Group spacing="sm">
        <Title order={2}>{title}</Title>
        {status}
      </Group>
      {description ? (
        <Text size="sm" color="dimmed">
          {description}
        </Text>
      ) : null}
    </Stack>
    {actions ? <Group spacing="xs">{actions}</Group> : null}
  </Group>
);
