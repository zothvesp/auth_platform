"use client";

import { Button, type ButtonProps } from "@mantine/core";
import { IconDeviceFloppy } from "@tabler/icons-react";
import type { ButtonHTMLAttributes, ReactNode } from "react";

type AppButtonVariant = "primary" | "secondary" | "danger" | "ghost";

export type AppButtonProps = Omit<ButtonProps, "variant" | "color"> &
  ButtonHTMLAttributes<HTMLButtonElement> & {
  appVariant?: AppButtonVariant;
  icon?: ReactNode;
};

const variantMap: Record<AppButtonVariant, Pick<ButtonProps, "variant" | "color">> = {
  primary: { variant: "filled", color: "cyan" },
  secondary: { variant: "default", color: "gray" },
  danger: { variant: "outline", color: "red" },
  ghost: { variant: "subtle", color: "gray" },
};

export const AppButton = ({
  appVariant = "primary",
  icon,
  children,
  ...props
}: AppButtonProps) => {
  const visual = variantMap[appVariant];

  return (
    <Button
      radius="md"
      size="xs"
      leftIcon={icon}
      styles={(theme) => ({
        root: {
          fontWeight: 600,
          letterSpacing: 0.2,
          "&:focus": {
            outline: "none",
            boxShadow: `0 0 0 2px ${theme.fn.rgba(theme.colors.cyan[4], 0.55)}`,
          },
          "&:disabled": {
            cursor: "default",
            opacity: 0.55,
          },
        },
      })}
      {...visual}
      {...props}
    >
      {children}
    </Button>
  );
};

export const SaveButton = (props: Omit<AppButtonProps, "icon" | "type">) => (
  <AppButton type="submit" icon={<IconDeviceFloppy size={15} />} {...props}>
    {props.children ?? "Save"}
  </AppButton>
);
