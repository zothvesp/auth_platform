"use client";

import { Alert, Box, Center, Paper } from "@mantine/core";
import type { AuthProps } from "@refinedev/mantine";
import { AuthPage as AuthPageBase } from "@refinedev/mantine";
import { IconInfoCircle } from "@tabler/icons-react";

export const AuthPage = (props: AuthProps) => {
  return (
    <AuthPageBase
      {...props}
      renderContent={(content) => (
        <Center mih="100vh" px="md">
          <Box maw={420} w="100%">
            <Alert
              icon={<IconInfoCircle size={16} />}
              color="cyan"
              variant="light"
              mb="md"
            >
              email: demo@refine.dev
              <br /> password: demodemo
            </Alert>
            <Paper radius="md" shadow="md" withBorder>
          {content}
            </Paper>
          </Box>
        </Center>
      )}
    />
  );
};
