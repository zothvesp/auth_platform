"use client";

import { Badge, Group, Stack, Text, Timeline } from "@mantine/core";
import type { ReactNode } from "react";

export type AuditTimelineItem = {
  action: string;
  actor?: string;
  details?: ReactNode;
  id: string;
  target: string;
  timestamp?: string;
};

type AuditTimelineProps = {
  items: AuditTimelineItem[];
};

export const AuditTimeline = ({ items }: AuditTimelineProps) => (
  <Timeline bulletSize={10} lineWidth={1} color="cyan">
    {items.map((item) => (
      <Timeline.Item
        key={item.id}
        title={
          <Group spacing="xs">
            <Badge size="sm" variant="light" color="cyan">
              {item.action}
            </Badge>
            <Text size="sm" weight={600}>
              {item.target}
            </Text>
          </Group>
        }
      >
        <Stack spacing={3} mt={4}>
          {item.details ? (
            <Text size="sm" color="dimmed">
              {item.details}
            </Text>
          ) : null}
          <Text size="xs" color="dimmed">
            {[item.actor, item.timestamp].filter(Boolean).join(" · ")}
          </Text>
        </Stack>
      </Timeline.Item>
    ))}
  </Timeline>
);
