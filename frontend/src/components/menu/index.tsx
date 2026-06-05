"use client";

import {
  Avatar,
  Box,
  Button,
  Divider,
  Group,
  Navbar,
  ScrollArea,
  Stack,
  Text,
  ThemeIcon,
  UnstyledButton,
} from "@mantine/core";
import { useLogout, useMenu } from "@refinedev/core";
import {
  IconAdjustments,
  IconClipboardList,
  IconDatabase,
  IconLogout,
  IconLockAccess,
  IconShield,
  IconUsers,
} from "@tabler/icons-react";
import Link from "next/link";

const icons: Record<string, React.ReactNode> = {
  audit_logs: <IconClipboardList size={16} />,
  permissions: <IconLockAccess size={16} />,
  roles: <IconShield size={16} />,
  settings: <IconAdjustments size={16} />,
  users: <IconUsers size={16} />,
};

export const Menu = () => {
  const { mutate: logout } = useLogout();
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
                    color:
                      active
                        ? theme.colors.cyan[theme.colorScheme === "dark" ? 3 : 7]
                        : theme.colorScheme === "dark"
                          ? theme.colors.gray[5]
                          : theme.colors.gray[7],
                    backgroundColor: active
                      ? theme.fn.rgba(
                          theme.colors.cyan[theme.colorScheme === "dark" ? 8 : 1],
                          theme.colorScheme === "dark" ? 0.25 : 0.7,
                        )
                      : "transparent",
                    "&:hover": {
                      backgroundColor:
                        theme.colorScheme === "dark" ? theme.colors.dark[5] : theme.colors.gray[1],
                      color: theme.colorScheme === "dark" ? theme.white : theme.black,
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

      <Navbar.Section>
        <Divider my="sm" />
        <Group position="apart" noWrap>
          <Group spacing="sm" noWrap>
            <Avatar radius="xl" color="cyan" size="sm">
              SA
            </Avatar>
            <Box>
              <Text size="xs" weight={600}>
                Sarah Admin
              </Text>
              <Text size={10} color="dimmed">
                SUPER_ADMIN
              </Text>
            </Box>
          </Group>
          <Button
            size="xs"
            compact
            variant="subtle"
            color="gray"
            leftIcon={<IconLogout size={14} />}
            onClick={() => logout()}
          >
            Logout
          </Button>
        </Group>
      </Navbar.Section>
    </Navbar>
  );
};
