"use client";

import { Skeleton, Stack } from "@mantine/core";

type TableSkeletonProps = {
  rows?: number;
};

export const TableSkeleton = ({ rows = 8 }: TableSkeletonProps) => (
  <Stack spacing="xs">
    {Array.from({ length: rows }).map((_, index) => (
      <Skeleton key={index} height={34} radius="md" />
    ))}
  </Stack>
);
