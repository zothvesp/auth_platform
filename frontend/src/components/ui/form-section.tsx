"use client";

import { Box, Divider, Group, Stack, Text } from "@mantine/core";
import type { ReactNode } from "react";

type FormSectionProps = {
  action?: ReactNode;
  children: ReactNode;
  description?: ReactNode;
  title: string;
};

export const FormSection = ({ action, children, description, title }: FormSectionProps) => (
  <Stack spacing="md">
    <Group position="apart" align="flex-start">
      <Box>
        <Text size="xs" weight={700} transform="uppercase" color="dimmed">
          {title}
        </Text>
        {description ? (
          <Text size="xs" color="dimmed" mt={3}>
            {description}
          </Text>
        ) : null}
      </Box>
      {action}
    </Group>
    <Divider />
    {children}
  </Stack>
);
