"use client";

import { Group, Stack, Text, ThemeIcon } from "@mantine/core";
import type { ReactNode } from "react";
import { SurfaceCard } from "./surface-card";

type MetricCardProps = {
  change?: ReactNode;
  icon?: ReactNode;
  label: string;
  value: ReactNode;
};

export const MetricCard = ({ change, icon, label, value }: MetricCardProps) => (
  <SurfaceCard>
    <Stack spacing="sm" p="md">
      <Group position="apart">
        <Text size="xs" weight={700} color="dimmed" transform="uppercase">
          {label}
        </Text>
        {icon ? (
          <ThemeIcon color="cyan" variant="light" radius="md">
            {icon}
          </ThemeIcon>
        ) : null}
      </Group>
      <Text size={28} weight={700}>
        {value}
      </Text>
      {change ? (
        <Text size="xs" color="dimmed">
          {change}
        </Text>
      ) : null}
    </Stack>
  </SurfaceCard>
);

type InfoCardProps = {
  action?: ReactNode;
  description?: ReactNode;
  icon?: ReactNode;
  title: ReactNode;
};

export const InfoCard = ({ action, description, icon, title }: InfoCardProps) => (
  <SurfaceCard>
    <Group align="flex-start" p="md" noWrap>
      {icon ? (
        <ThemeIcon color="cyan" variant="light" radius="md" size={36}>
          {icon}
        </ThemeIcon>
      ) : null}
      <Stack spacing={4} sx={{ flex: 1 }}>
        <Text size="sm" weight={700}>
          {title}
        </Text>
        {description ? (
          <Text size="sm" color="dimmed">
            {description}
          </Text>
        ) : null}
        {action}
      </Stack>
    </Group>
  </SurfaceCard>
);
