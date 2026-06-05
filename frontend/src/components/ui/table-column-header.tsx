"use client";

import { Group, Text, Tooltip } from "@mantine/core";
import { IconArrowsSort, IconChevronDown, IconChevronUp, IconInfoCircle } from "@tabler/icons-react";
import type { ReactNode } from "react";

type SortDirection = false | "asc" | "desc";

type TableColumnHeaderProps = {
  helper?: ReactNode;
  label: ReactNode;
  onSort?: () => void;
  sortable?: boolean;
  sorted?: SortDirection;
};

export const TableColumnHeader = ({
  helper,
  label,
  onSort,
  sortable,
  sorted = false,
}: TableColumnHeaderProps) => (
  <Group
    spacing={6}
    noWrap
    onClick={sortable ? onSort : undefined}
    sx={{ cursor: sortable ? "pointer" : "default" }}
  >
    <Text size="xs" weight={700} transform="uppercase" color="dimmed">
      {label}
    </Text>
    {helper ? (
      <Tooltip label={helper} withArrow>
        <IconInfoCircle size={13} opacity={0.6} />
      </Tooltip>
    ) : null}
    {sortable ? (
      sorted === "asc" ? (
        <IconChevronUp size={14} />
      ) : sorted === "desc" ? (
        <IconChevronDown size={14} />
      ) : (
        <IconArrowsSort size={14} opacity={0.45} />
      )
    ) : null}
  </Group>
);
