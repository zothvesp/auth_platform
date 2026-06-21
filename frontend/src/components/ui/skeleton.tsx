"use client";

import {
  Box,
  Group,
  Paper,
  SimpleGrid,
  Skeleton,
  Stack,
  Table,
} from "@mantine/core";

type SkeletonProps = {
  width?: number | string;
  height?: number | string;
  radius?: "xs" | "sm" | "md" | "lg" | "xl" | number;
};

const Line = ({ width = "100%", height = 14, radius = "sm" }: SkeletonProps) => (
  <Skeleton width={width} height={height} radius={radius} />
);

// ─── PageSkeleton ─────────────────────────────────────────────────────────────
// Mimics PageHeader layout: title + description + optional actions

export const PageSkeleton = () => (
  <Stack spacing="xs" mb="lg">
    <Line width="35%" height={28} />
    <Line width="55%" height={14} />
  </Stack>
);

// ─── TableSkeleton ────────────────────────────────────────────────────────────
// Mimics DataTable with header row + N placeholder rows

const TABLE_ROWS = 8;

export const TableSkeleton = ({ rows = TABLE_ROWS }: { rows?: number }) => (
  <Paper withBorder p={0}>
    <Table>
      <thead>
        <tr>
          {Array.from({ length: 5 }).map((_, i) => (
            <th key={i}>
              <Line width="70%" height={12} />
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {Array.from({ length: rows }).map((_, row) => (
          <tr key={row}>
            {Array.from({ length: 5 }).map((_, cell) => (
              <td key={cell}>
                <Line
                  width={cell === 0 ? "60%" : cell === 4 ? "30%" : "80%"}
                  height={14}
                />
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </Table>
  </Paper>
);

// ─── CardSkeleton ─────────────────────────────────────────────────────────────
// Mimics SurfaceCard with skeleton content inside

export const CardSkeleton = ({ lines = 3 }: { lines?: number }) => (
  <Paper withBorder p="md">
    <Stack spacing="sm">
      <Line width="40%" height={18} />
      {Array.from({ length: lines }).map((_, i) => (
        <Line key={i} width={i === lines - 1 ? "65%" : "100%"} height={14} />
      ))}
    </Stack>
  </Paper>
);

// ─── FormSkeleton ─────────────────────────────────────────────────────────────
// Mimics form field layout: label + input pairs

const FORM_FIELDS = 4;

export const FormSkeleton = ({ fields = FORM_FIELDS }: { fields?: number }) => (
  <Stack spacing="lg">
    {Array.from({ length: fields }).map((_, i) => (
      <Stack key={i} spacing={4}>
        <Line width="25%" height={12} />
        <Skeleton height={36} radius="sm" />
      </Stack>
    ))}
  </Stack>
);

// ─── DetailGridSkeleton ───────────────────────────────────────────────────────
// Mimics DetailGrid columns with label/value blocks

const GRID_ITEMS = 9;

export const DetailGridSkeleton = ({
  columns = 3,
  items = GRID_ITEMS,
}: {
  columns?: number;
  items?: number;
}) => (
  <Paper withBorder p="md">
    <SimpleGrid
      cols={columns}
      breakpoints={[
        { maxWidth: "md", cols: Math.min(columns, 2) },
        { maxWidth: "sm", cols: 1 },
      ]}
      spacing={0}
    >
      {Array.from({ length: items }).map((_, i) => (
        <Box
          key={i}
          p="sm"
          sx={(theme) => ({
            borderBottom: `1px solid ${theme.colors.dark[4]}`,
          })}
        >
          <Stack spacing={3}>
            <Line width="50%" height={10} />
            <Line width="70%" height={14} />
          </Stack>
        </Box>
      ))}
    </SimpleGrid>
  </Paper>
);
