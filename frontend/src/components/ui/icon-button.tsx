"use client";

import { ActionIcon, Tooltip, type ActionIconProps } from "@mantine/core";
import type { ButtonHTMLAttributes, ReactNode } from "react";

export type AppIconButtonProps = ActionIconProps &
  ButtonHTMLAttributes<HTMLButtonElement> & {
  label: string;
  icon: ReactNode;
};

export const AppIconButton = ({ label, icon, styles: userStyles, ...props }: AppIconButtonProps) => {
  return (
    <Tooltip label={label} withArrow>
      <ActionIcon
        aria-label={label}
        radius="md"
        variant="subtle"
        color="gray"
        styles={(theme) => ({
          root: {
            "&:hover": {
              backgroundColor: theme.colors.dark[5],
              color: theme.white,
            },
            "&:focus": {
              outline: "none",
              boxShadow: `0 0 0 2px ${theme.fn.rgba(theme.colors.cyan[4], 0.55)}`,
            },
          },
          ...userStyles,
        })}
        {...props}
      >
        {icon}
      </ActionIcon>
    </Tooltip>
  );
};
