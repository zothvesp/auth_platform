"use client";

import { Group, Paper, Text } from "@mantine/core";
import type { ReactNode } from "react";

type FilterBarProps = {
  children: ReactNode;
  label?: string;
};

export const FilterBar = ({ children, label = "Filters" }: FilterBarProps) => (
  <Paper withBorder radius="md" p="xs">
    <Group spacing="sm" align="center">
      <Text size="xs" weight={700} color="dimmed" transform="uppercase">
        {label}
      </Text>
      <Group spacing="xs">{children}</Group>
    </Group>
  </Paper>
);
