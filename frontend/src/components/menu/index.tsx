"use client";

import {
  Box,
  Divider,
  Group,
  Navbar,
  ScrollArea,
  Stack,
  Text,
  ThemeIcon,
  UnstyledButton,
} from "@mantine/core";
import { useMenu } from "@refinedev/core";
import {
  IconAdjustments,
  IconClipboardList,
  IconDatabase,
  IconLockAccess,
  IconKey,
  IconShield,
  IconUserCircle,
  IconUsers,
} from "@tabler/icons-react";
import Link from "next/link";

const icons: Record<string, React.ReactNode> = {
  audit_logs: <IconClipboardList size={16} />,
  permissions: <IconLockAccess size={16} />,
  profile: <IconUserCircle size={16} />,
  roles: <IconShield size={16} />,
  security: <IconKey size={16} />,
  settings: <IconAdjustments size={16} />,
  users: <IconUsers size={16} />,
};

export const Menu = () => {
  const { menuItems, selectedKey } = useMenu();

  return (
    <Navbar width={{ base: 260 }} p="sm">
      <Navbar.Section>
        <Group spacing="sm" px="xs" py="sm">
          <ThemeIcon size={34} radius="md" variant="gradient">
            <IconDatabase size={18} />
          </ThemeIcon>
          <Box>
            <Text size="sm" weight={700}>
              OPSCTRL
            </Text>
            <Text size={10} color="dimmed" transform="uppercase">
              Admin Console
            </Text>
          </Box>
        </Group>
      </Navbar.Section>

      <Divider my="sm" />

      <Navbar.Section grow>
        <ScrollArea h="100%">
          <Stack spacing={4}>
            {menuItems.map((item) => (
              <UnstyledButton
                key={item.key}
                component={Link}
                href={item.route ?? "/"}
                sx={(theme) => {
                  const active = selectedKey === item.key;

                  return {
                    display: "flex",
                    alignItems: "center",
                    gap: theme.spacing.sm,
                    width: "100%",
                    padding: `${theme.spacing.xs}px ${theme.spacing.sm}px`,
                    borderRadius: theme.radius.md,
                    color: active
                      ? theme.colors.cyan[3]
                      : theme.colors.gray[5],
                    backgroundColor: active
                      ? theme.fn.rgba(theme.colors.cyan[8], 0.25)
                      : "transparent",
                    "&:hover": {
                      backgroundColor: theme.colors.dark[5],
                      color: theme.white,
                    },
                  };
                }}
              >
                <ThemeIcon size={26} radius="md" variant="light" color="cyan">
                  {icons[item.name] ?? <IconDatabase size={16} />}
                </ThemeIcon>
                <Text size="sm">{item.label}</Text>
              </UnstyledButton>
            ))}
          </Stack>
        </ScrollArea>
      </Navbar.Section>
    </Navbar>
  );
};
