"use client";

import { Stack, Text } from "@mantine/core";
import type { ReactNode } from "react";

export type KeyValueItem = {
  label: string;
  value: ReactNode;
};

type KeyValueListProps = {
  items: KeyValueItem[];
};

export const KeyValueList = ({ items }: KeyValueListProps) => (
  <Stack spacing={0}>
    {items.map((item) => (
      <Stack
        key={item.label}
        spacing={2}
        py="xs"
        sx={(theme) => ({ borderBottom: `1px solid ${theme.colors.dark[4]}` })}
      >
        <Text size="xs" color="dimmed" weight={700} transform="uppercase">
          {item.label}
        </Text>
        <Text size="sm">{item.value}</Text>
      </Stack>
    ))}
  </Stack>
);
