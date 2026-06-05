"use client";

import { Checkbox, Group, Stack, Text } from "@mantine/core";
import { SurfaceCard } from "./surface-card";

export type PermissionMatrixGroup = {
  description?: string;
  permissions: Array<{
    description?: string;
    label: string;
    value: string;
  }>;
  title: string;
};

type PermissionMatrixProps = {
  groups: PermissionMatrixGroup[];
  onChange: (values: string[]) => void;
  value: string[];
};

export const PermissionMatrix = ({ groups, onChange, value }: PermissionMatrixProps) => {
  const valueSet = new Set(value);

  const toggle = (permission: string) => {
    const next = new Set(valueSet);
    if (next.has(permission)) next.delete(permission);
    else next.add(permission);
    onChange(Array.from(next));
  };

  const toggleGroup = (permissions: string[], checked: boolean) => {
    const next = new Set(valueSet);
    permissions.forEach((permission) => {
      if (checked) next.add(permission);
      else next.delete(permission);
    });
    onChange(Array.from(next));
  };

  return (
    <Stack spacing="sm">
      {groups.map((group) => {
        const groupValues = group.permissions.map((permission) => permission.value);
        const selected = groupValues.filter((permission) => valueSet.has(permission));
        const allSelected = selected.length === groupValues.length;
        const someSelected = selected.length > 0 && !allSelected;

        return (
          <SurfaceCard key={group.title}>
            <Stack spacing="sm" p="md">
              <Group position="apart" align="flex-start">
                <div>
                  <Text size="sm" weight={700}>
                    {group.title}
                  </Text>
                  {group.description ? (
                    <Text size="xs" color="dimmed">
                      {group.description}
                    </Text>
                  ) : null}
                </div>
                <Checkbox
                  color="cyan"
                  checked={allSelected}
                  indeterminate={someSelected}
                  label="Select all"
                  onChange={(event) => toggleGroup(groupValues, event.currentTarget.checked)}
                />
              </Group>
              <Stack spacing="xs">
                {group.permissions.map((permission) => (
                  <Checkbox
                    key={permission.value}
                    color="cyan"
                    checked={valueSet.has(permission.value)}
                    label={
                      <div>
                        <Text size="sm">{permission.label}</Text>
                        {permission.description ? (
                          <Text size="xs" color="dimmed">
                            {permission.description}
                          </Text>
                        ) : null}
                      </div>
                    }
                    onChange={() => toggle(permission.value)}
                  />
                ))}
              </Stack>
            </Stack>
          </SurfaceCard>
        );
      })}
    </Stack>
  );
};
