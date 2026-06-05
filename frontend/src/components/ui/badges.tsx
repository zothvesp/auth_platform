"use client";

import { Badge, type BadgeProps } from "@mantine/core";

export type Classification = "restricted" | "confidential" | "internal" | "public";
export type RecordStatus = "active" | "inactive" | "suspended" | "draft" | "published" | "rejected";

const classificationColors: Record<Classification, BadgeProps["color"]> = {
  restricted: "red",
  confidential: "yellow",
  internal: "cyan",
  public: "green",
};

const statusColors: Record<RecordStatus, BadgeProps["color"]> = {
  active: "green",
  inactive: "gray",
  suspended: "yellow",
  draft: "gray",
  published: "green",
  rejected: "red",
};

export const ClassificationBadge = ({ value }: { value: Classification }) => (
  <Badge color={classificationColors[value]} variant="light" radius="sm" size="sm">
    {value}
  </Badge>
);

export const StatusBadge = ({ value }: { value: RecordStatus | string }) => {
  const color = value in statusColors ? statusColors[value as RecordStatus] : "gray";

  return (
    <Badge color={color} variant="light" radius="xl" size="sm">
      {value}
    </Badge>
  );
};
