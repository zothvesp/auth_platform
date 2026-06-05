"use client";

import { Menu } from "@mantine/core";
import {
  IconArchive,
  IconDotsVertical,
  IconEye,
  IconPencil,
  IconTrash,
} from "@tabler/icons-react";
import type { ReactNode } from "react";
import { AppIconButton } from "./icon-button";

export type RowAction = {
  color?: string;
  disabled?: boolean;
  icon?: ReactNode;
  label: string;
  onClick: () => void;
};

type RowActionsMenuProps = {
  actions?: RowAction[];
  extra?: ReactNode;
  onArchive?: () => void;
  onDelete?: () => void;
  onEdit?: () => void;
  onView?: () => void;
};

export const RowActionsMenu = ({
  actions = [],
  extra,
  onArchive,
  onDelete,
  onEdit,
  onView,
}: RowActionsMenuProps) => {
  const builtIn: RowAction[] = [
    onView ? { icon: <IconEye size={15} />, label: "View", onClick: onView } : null,
    onEdit ? { icon: <IconPencil size={15} />, label: "Edit", onClick: onEdit } : null,
    onArchive ? { icon: <IconArchive size={15} />, label: "Archive", onClick: onArchive } : null,
    onDelete
      ? { color: "red", icon: <IconTrash size={15} />, label: "Delete", onClick: onDelete }
      : null,
  ].filter(Boolean) as RowAction[];

  return (
    <Menu shadow="md" width={180} position="bottom-end">
      <Menu.Target>
        <AppIconButton label="Row actions" icon={<IconDotsVertical size={16} />} />
      </Menu.Target>
      <Menu.Dropdown>
        {[...builtIn, ...actions].map((action) => (
          <Menu.Item
            key={action.label}
            color={action.color}
            disabled={action.disabled}
            icon={action.icon}
            onClick={action.onClick}
          >
            {action.label}
          </Menu.Item>
        ))}
        {extra ? (
          <>
            <Menu.Divider />
            {extra}
          </>
        ) : null}
      </Menu.Dropdown>
    </Menu>
  );
};
