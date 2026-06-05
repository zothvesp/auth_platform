"use client";

import { Card, Group, Text, type CardProps } from "@mantine/core";
import type { ReactNode } from "react";

type SurfaceCardProps = CardProps & {
  title?: string;
  description?: string;
  action?: ReactNode;
};

export const SurfaceCard = ({
  title,
  description,
  action,
  children,
  ...props
}: SurfaceCardProps) => (
  <Card withBorder radius="md" p={0} {...props}>
    {title || description || action ? (
      <Card.Section withBorder px="md" py="sm">
        <Group position="apart" align="flex-start">
          <div>
            {title ? (
              <Text size="xs" weight={700} transform="uppercase" color="dimmed">
                {title}
              </Text>
            ) : null}
            {description ? (
              <Text size="xs" color="dimmed" mt={2}>
                {description}
              </Text>
            ) : null}
          </div>
          {action}
        </Group>
      </Card.Section>
    ) : null}
    {children}
  </Card>
);
