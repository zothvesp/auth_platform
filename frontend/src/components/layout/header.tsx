"use client";

import {
  ActionIcon,
  Avatar,
  Badge,
  Box,
  Group,
  Menu,
  Divider,
  Text,
  UnstyledButton,
} from "@mantine/core";
import {
  IconBell,
  IconLogout,
  IconSettings,
  IconUserCircle,
  IconShield,
  IconKey,
} from "@tabler/icons-react";
import Link from "next/link";
import { useEffect, useState } from "react";
import type { AuthUser } from "@lib/auth-api";
import { useTranslations } from "@lib/i18n";
import { apiRequest } from "@lib/request";
import { useLogout } from "@refinedev/core";

export const HeaderContent = () => {
  const { mutate: logout } = useLogout();
  const t = useTranslations();
  const [user, setUser] = useState<AuthUser>();
  const [notifications, setNotifications] = useState(0);

  useEffect(() => {
    apiRequest<AuthUser>("/users/me")
      .then(setUser)
      .catch(() => {});
  }, []);

  const roleLabel = user?.roles?.[0]?.name ?? "user";
  const initials = user?.displayName
    ?.split(" ")
    .map((n) => n[0])
    .join("")
    .toUpperCase()
    .slice(0, 2) ?? "U";

  return (
    <Group spacing="xs" noWrap>
      {/* Notifications */}
      <Menu shadow="md" width={320} position="bottom-end" withinPortal>
        <Menu.Target>
          <UnstyledButton>
            <ActionIcon variant="subtle" color="gray" size="lg" radius="md">
              <Box pos="relative">
                <IconBell size={20} />
                {notifications > 0 && (
                  <Badge
                    color="red"
                    size="xs"
                    variant="filled"
                    p={0}
                    w={16}
                    h={16}
                    sx={{ position: "absolute", top: -6, right: -8 }}
                  >
                    {notifications > 99 ? "99+" : notifications}
                  </Badge>
                )}
              </Box>
            </ActionIcon>
          </UnstyledButton>
        </Menu.Target>
        <Menu.Dropdown>
          <Menu.Label>Notifications</Menu.Label>
          <Menu.Item>
            <Text size="sm" color="dimmed">
              No new notifications
            </Text>
          </Menu.Item>
        </Menu.Dropdown>
      </Menu>

      <Divider orientation="vertical" mx={4} />

      {/* User profile dropdown */}
      <Menu shadow="md" width={220} position="bottom-end" withinPortal>
        <Menu.Target>
          <UnstyledButton>
            <Group spacing="sm" noWrap>
              <Avatar radius="xl" color="cyan" size={32}>
                {initials}
              </Avatar>
              <Box sx={{ display: "none" }} mah={40}>
                <Text size="xs" weight={600} lineH={1.2}>
                  {user?.displayName ?? "..."}
                </Text>
                <Text size={10} color="dimmed" lineH={1.2}>
                  {roleLabel}
                </Text>
              </Box>
            </Group>
          </UnstyledButton>
        </Menu.Target>
        <Menu.Dropdown>
          <Menu.Label>
            <Group spacing="sm">
              <Avatar radius="xl" color="cyan" size={36}>
                {initials}
              </Avatar>
              <Box>
                <Text size="sm" weight={600}>
                  {user?.displayName ?? "Loading..."}
                </Text>
                <Text size="xs" color="dimmed">
                  {user?.email ?? ""}
                </Text>
              </Box>
            </Group>
          </Menu.Label>
          <Divider />
          <Menu.Item
            icon={<IconUserCircle size={16} />}
            component={Link}
            href="/profile"
          >
            {t.pages?.users?.profile ?? "My Profile"}
          </Menu.Item>
          <Menu.Item
            icon={<IconKey size={16} />}
            component={Link}
            href="/security"
          >
            {t.pages?.security?.title ?? "Security"}
          </Menu.Item>
          {user?.roles?.some((r) =>
            ["super_admin", "admin"].includes(r.name),
          ) && (
            <Menu.Item
              icon={<IconSettings size={16} />}
              component={Link}
              href="/settings"
            >
              {t.pages?.settings?.title ?? "Settings"}
            </Menu.Item>
          )}
          <Menu.Divider />
          <Menu.Item
            icon={<IconLogout size={16} />}
            color="red"
            onClick={() => logout()}
          >
            {t.common.logout}
          </Menu.Item>
        </Menu.Dropdown>
      </Menu>
    </Group>
  );
};
