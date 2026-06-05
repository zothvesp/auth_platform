"use client";

import { SimpleGrid, Stack, Text } from "@mantine/core";
import type { ReactNode } from "react";
import { SurfaceCard } from "./surface-card";

export type DetailGridItem = {
  label: string;
  value: ReactNode;
};

type DetailGridProps = {
  columns?: 1 | 2 | 3 | 4;
  items: DetailGridItem[];
  title?: string;
};

export const DetailGrid = ({ columns = 3, items, title }: DetailGridProps) => (
  <SurfaceCard title={title}>
    <SimpleGrid
      cols={columns}
      breakpoints={[
        { maxWidth: "md", cols: Math.min(columns, 2) },
        { maxWidth: "sm", cols: 1 },
      ]}
      spacing={0}
      p="md"
    >
      {items.map((item) => (
        <Stack
          key={item.label}
          spacing={3}
          p="sm"
          sx={(theme) => ({ borderBottom: `1px solid ${theme.colors.dark[4]}` })}
        >
          <Text size="xs" color="dimmed" weight={700} transform="uppercase">
            {item.label}
          </Text>
          <Text size="sm">{item.value}</Text>
        </Stack>
      ))}
    </SimpleGrid>
  </SurfaceCard>
);
