"use client";

import { Group, Paper, type GroupProps } from "@mantine/core";

export const ActionRow = (props: GroupProps) => (
  <Paper withBorder radius="md" p="xs">
    <Group spacing="xs" {...props} />
  </Paper>
);
