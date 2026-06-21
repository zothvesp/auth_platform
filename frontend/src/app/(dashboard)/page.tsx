"use client";

import {
  Box,
  Card,
  Grid,
  Group,
  Loader,
  SimpleGrid,
  Stack,
  Text,
  ThemeIcon,
} from "@mantine/core";
import {
  IconUsers,
  IconShield,
  IconClipboardList,
  IconKey,
  IconAlertTriangle,
  IconCheck,
} from "@tabler/icons-react";
import { useEffect, useState } from "react";
import { apiRequest } from "@lib/request";
import { useTranslations } from "@lib/i18n";

type Stats = {
  users: number;
  roles: number;
  permissions: number;
  auditLogs: number;
};

type HealthStatus = {
  status: string;
  database: string;
  redis: string;
};

const StatCard = ({
  label,
  value,
  icon,
  color,
}: {
  label: string;
  value: number | string;
  icon: React.ReactNode;
  color: string;
}) => (
  <Card padding="lg" radius="md">
    <Group position="apart" align="flex-start">
      <Box>
        <Text size="xs" color="dimmed" transform="uppercase" weight={600}>
          {label}
        </Text>
        <Text size="xl" weight={700} mt={4}>
          {value}
        </Text>
      </Box>
      <ThemeIcon size={42} radius="md" variant="light" color={color}>
        {icon}
      </ThemeIcon>
    </Group>
  </Card>
);

export default function DashboardHome() {
  const t = useTranslations();
  const [stats, setStats] = useState<Stats | null>(null);
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      apiRequest<{ total: number }>("/admin/users?page=1&page_size=1"),
      apiRequest<unknown[]>("/admin/roles"),
      apiRequest<unknown[]>("/admin/permissions"),
      apiRequest<unknown[]>("/admin/audit-logs?page=1&page_size=1"),
    ])
      .then(([users, roles, perms, logs]) => {
        setStats({
          users: users.total ?? 0,
          roles: roles.length ?? 0,
          permissions: perms.length ?? 0,
          auditLogs: logs.length ?? 0,
        });
      })
      .catch(() => {})
      .finally(() => setLoading(false));

    fetch("/health")
      .then((r) => r.json())
      .then((data) => setHealth(data))
      .catch(() => setHealth({ status: "unreachable", database: "unknown", redis: "unknown" }));
  }, []);

  if (loading) {
    return (
      <Group position="center" py="xl">
        <Loader />
      </Group>
    );
  }

  return (
    <Stack spacing="xl">
      <Box>
        <Text size="xl" weight={700}>
          Dashboard
        </Text>
        <Text size="sm" color="dimmed" mt={4}>
          System overview and recent activity
        </Text>
      </Box>

      {/* Stats grid */}
      <SimpleGrid cols={4} spacing="md" breakpoints={[
        { maxWidth: "md", cols: 2 },
        { maxWidth: "sm", cols: 1 },
      ]}>
        <StatCard
          label="Users"
          value={stats?.users ?? 0}
          icon={<IconUsers size={22} />}
          color="cyan"
        />
        <StatCard
          label="Roles"
          value={stats?.roles ?? 0}
          icon={<IconShield size={22} />}
          color="violet"
        />
        <StatCard
          label="Permissions"
          value={stats?.permissions ?? 0}
          icon={<IconKey size={22} />}
          color="teal"
        />
        <StatCard
          label="Audit Events"
          value={stats?.auditLogs ?? 0}
          icon={<IconClipboardList size={22} />}
          color="orange"
        />
      </SimpleGrid>

      {/* Health status */}
      <Card padding="lg" radius="md">
        <Text size="sm" weight={600} mb="md">
          System Health
        </Text>
        <SimpleGrid cols={3} spacing="md" breakpoints={[
          { maxWidth: "sm", cols: 1 },
        ]}>
          <Group spacing="sm">
            <ThemeIcon
              size={28}
              radius="xl"
              color={health?.status === "OK" ? "green" : "red"}
              variant="light"
            >
              {health?.status === "OK" ? (
                <IconCheck size={16} />
              ) : (
                <IconAlertTriangle size={16} />
              )}
            </ThemeIcon>
            <Box>
              <Text size="xs" color="dimmed">
                API Server
              </Text>
              <Text size="sm" weight={500}>
                {health?.status ?? "Unknown"}
              </Text>
            </Box>
          </Group>
          <Group spacing="sm">
            <ThemeIcon
              size={28}
              radius="xl"
              color={health?.database === "connected" ? "green" : "red"}
              variant="light"
            >
              {health?.database === "connected" ? (
                <IconCheck size={16} />
              ) : (
                <IconAlertTriangle size={16} />
              )}
            </ThemeIcon>
            <Box>
              <Text size="xs" color="dimmed">
                PostgreSQL
              </Text>
              <Text size="sm" weight={500}>
                {health?.database ?? "Unknown"}
              </Text>
            </Box>
          </Group>
          <Group spacing="sm">
            <ThemeIcon
              size={28}
              radius="xl"
              color={health?.redis === "connected" ? "green" : "red"}
              variant="light"
            >
              {health?.redis === "connected" ? (
                <IconCheck size={16} />
              ) : (
                <IconAlertTriangle size={16} />
              )}
            </ThemeIcon>
            <Box>
              <Text size="xs" color="dimmed">
                Redis
              </Text>
              <Text size="sm" weight={500}>
                {health?.redis ?? "Unknown"}
              </Text>
            </Box>
          </Group>
        </SimpleGrid>
      </Card>

      {/* Quick links */}
      <Card padding="lg" radius="md">
        <Text size="sm" weight={600} mb="md">
          Quick Actions
        </Text>
        <SimpleGrid cols={3} spacing="md" breakpoints={[
          { maxWidth: "sm", cols: 1 },
        ]}>
          <Card
            component="a"
            href="/users/create"
            padding="md"
            radius="md"
            withBorder
            sx={{ cursor: "pointer", textDecoration: "none" }}
          >
            <Group spacing="sm">
              <ThemeIcon size={32} radius="md" variant="light" color="cyan">
                <IconUsers size={18} />
              </ThemeIcon>
              <Text size="sm" weight={500}>
                Create User
              </Text>
            </Group>
          </Card>
          <Card
            component="a"
            href="/roles"
            padding="md"
            radius="md"
            withBorder
            sx={{ cursor: "pointer", textDecoration: "none" }}
          >
            <Group spacing="sm">
              <ThemeIcon size={32} radius="md" variant="light" color="violet">
                <IconShield size={18} />
              </ThemeIcon>
              <Text size="sm" weight={500}>
                Manage Roles
              </Text>
            </Group>
          </Card>
          <Card
            component="a"
            href="/settings"
            padding="md"
            radius="md"
            withBorder
            sx={{ cursor: "pointer", textDecoration: "none" }}
          >
            <Group spacing="sm">
              <ThemeIcon size={32} radius="md" variant="light" color="teal">
                <IconKey size={18} />
              </ThemeIcon>
              <Text size="sm" weight={500}>
                System Settings
              </Text>
            </Group>
          </Card>
        </SimpleGrid>
      </Card>
    </Stack>
  );
}
