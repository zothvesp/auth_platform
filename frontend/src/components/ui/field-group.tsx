"use client";

import { SimpleGrid, type SimpleGridProps } from "@mantine/core";

type FieldGroupProps = SimpleGridProps & {
  columns?: 1 | 2 | 3 | 4;
};

export const FieldGroup = ({ columns = 2, children, ...props }: FieldGroupProps) => (
  <SimpleGrid
    cols={columns}
    breakpoints={[
      { maxWidth: "md", cols: Math.min(columns, 2) },
      { maxWidth: "sm", cols: 1 },
    ]}
    spacing="md"
    {...props}
  >
    {children}
  </SimpleGrid>
);
