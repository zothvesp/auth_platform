"use client";

import { Group, Text } from "@mantine/core";

type StatusDotProps = {
  color?: string;
  label?: string;
};

export const StatusDot = ({ color = "var(--nsn-uif-refreshed-color-utility-success)", label }: StatusDotProps) => (
  <Group spacing={6} noWrap>
    <span
      aria-hidden
      style={{
        width: 7,
        height: 7,
        borderRadius: 999,
        background: color,
        boxShadow: `0 0 6px ${color}`,
        display: "inline-block",
      }}
    />
    {label ? (
      <Text size="xs" color="dimmed">
        {label}
      </Text>
    ) : null}
  </Group>
);
