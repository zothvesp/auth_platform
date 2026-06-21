"use client";

import { Anchor, Box, Center, Group, Stack, Text, ThemeIcon, Title } from "@mantine/core";
import { IconShieldLock } from "@tabler/icons-react";
import Link from "next/link";
import type { ReactNode } from "react";
import { SurfaceCard } from "@components/ui";
import { useTranslations } from "@lib/i18n";

type AuthShellProps = {
  children: ReactNode;
  description: string;
  footer?: ReactNode;
  title: string;
};

export const AuthShell = ({ children, description, footer, title }: AuthShellProps) => {
  const t = useTranslations();
  return (
    <Center mih="100vh" px="md" py="xl">
      <Stack spacing="lg" maw={440} w="100%">
        <Stack spacing="xs" align="center">
          <ThemeIcon size={44} radius="md" color="cyan" variant="light">
            <IconShieldLock size={24} />
          </ThemeIcon>
          <Box ta="center">
            <Title order={2}>{t.authShell.brand}</Title>
            <Text size="sm" color="dimmed">
              {t.authShell.tagline}
            </Text>
          </Box>
        </Stack>

      <SurfaceCard>
        <Stack spacing="md" p="lg">
          <Stack spacing={3}>
            <Title order={3}>{title}</Title>
            <Text size="sm" color="dimmed">
              {description}
            </Text>
          </Stack>
          {children}
        </Stack>
      </SurfaceCard>

      {footer ? (
        <Group position="center">
          <Text size="sm" color="dimmed">
            {footer}
          </Text>
        </Group>
      ) : null}
    </Stack>
  </Center>
  );
};

export const AuthLink = ({ children, href }: { children: ReactNode; href: string }) => (
  <Anchor component={Link} href={href} size="sm">
    {children}
  </Anchor>
);
