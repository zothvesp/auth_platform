"use client";

import { Alert, Center, Stack, Text, Title } from "@mantine/core";
import { IconAlertCircle, IconDatabaseOff } from "@tabler/icons-react";
import type { ReactNode } from "react";

type EmptyStateProps = {
  action?: ReactNode;
  description?: ReactNode;
  icon?: ReactNode;
  title?: string;
};

export const EmptyState = ({
  action,
  description = "Try changing filters or creating a new record.",
  icon = <IconDatabaseOff size={38} />,
  title = "No records found",
}: EmptyStateProps) => (
  <Center py="xl">
    <Stack align="center" spacing="xs" maw={420}>
      {icon}
      <Title order={4}>{title}</Title>
      <Text size="sm" color="dimmed" align="center">
        {description}
      </Text>
      {action}
    </Stack>
  </Center>
);

type ErrorStateProps = {
  action?: ReactNode;
  message?: ReactNode;
  title?: string;
};

export const ErrorState = ({
  action,
  message = "The request could not be completed. Please try again.",
  title = "Something went wrong",
}: ErrorStateProps) => (
  <Alert icon={<IconAlertCircle size={18} />} title={title} color="red" variant="light">
    <Stack spacing="sm">
      <Text size="sm">{message}</Text>
      {action}
    </Stack>
  </Alert>
);
