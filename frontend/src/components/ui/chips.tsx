"use client";

import { Group, Tooltip } from "@mantine/core";
import type { ReactNode } from "react";

type ChipListProps = {
  color?: string;
  items: Array<string | { label: string; tooltip?: ReactNode }>;
  limit?: number;
};

export const ChipList = ({ color = "cyan", items, limit = 3 }: ChipListProps) => {
  const visible = items.slice(0, limit);
  const hidden = items.slice(limit);

  return (
    <Group spacing={4}>
      {visible.map((item) => {
        const label = typeof item === "string" ? item : item.label;

        return typeof item === "string" || !item.tooltip ? (
          <span
            key={label}
            className="chip"
            style={{
              borderColor: `var(--mantine-color-${color}-5)`,
              color: `var(--mantine-color-${color}-3)`,
            }}
          >
            {label}
          </span>
        ) : (
          <Tooltip key={label} label={item.tooltip}>
            <span
              className="chip"
              style={{
                borderColor: `var(--mantine-color-${color}-5)`,
                color: `var(--mantine-color-${color}-3)`,
              }}
            >
              {label}
            </span>
          </Tooltip>
        );
      })}
      {hidden.length ? (
        <Tooltip label={hidden.map((item) => (typeof item === "string" ? item : item.label)).join(", ")}>
          <span className="chip">+{hidden.length}</span>
        </Tooltip>
      ) : null}
    </Group>
  );
};
