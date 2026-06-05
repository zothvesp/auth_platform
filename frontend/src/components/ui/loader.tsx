"use client";

import { Center, Loader, Stack, Text } from "@mantine/core";

export const AppLoader = ({ label = "Loading" }: { label?: string }) => (
  <Center py="xl">
    <Stack spacing="xs" align="center">
      <Loader color="cyan" size="sm" />
      <Text size="xs" color="dimmed">
        {label}
      </Text>
    </Stack>
  </Center>
);
