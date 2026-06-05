"use client";

import { Group, Stack, Text } from "@mantine/core";
import type { ReactNode } from "react";
import { SurfaceCard } from "./surface-card";

type DangerZoneProps = {
  action: ReactNode;
  description: ReactNode;
  title?: string;
};

export const DangerZone = ({ action, description, title = "Danger zone" }: DangerZoneProps) => (
  <SurfaceCard>
    <Group position="apart" align="center" p="md">
      <Stack spacing={3}>
        <Text size="sm" weight={700} color="red">
          {title}
        </Text>
        <Text size="sm" color="dimmed">
          {description}
        </Text>
      </Stack>
      {action}
    </Group>
  </SurfaceCard>
);
