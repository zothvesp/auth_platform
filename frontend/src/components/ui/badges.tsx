"use client";

import { Badge, type BadgeProps } from "@mantine/core";

export type RecordStatus = "active" | "inactive" | "suspended" | "draft" | "published" | "rejected";

const statusColors: Record<RecordStatus, BadgeProps["color"]> = {
  active: "green",
  inactive: "gray",
  suspended: "yellow",
  draft: "gray",
  published: "green",
  rejected: "red",
};

export const StatusBadge = ({ value }: { value: RecordStatus | string }) => {
  const color = value in statusColors ? statusColors[value as RecordStatus] : "gray";

  return (
    <Badge color={color} variant="light" radius="xl" size="sm">
      {value}
    </Badge>
  );
};
