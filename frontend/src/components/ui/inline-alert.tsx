"use client";

import { Alert, type AlertProps } from "@mantine/core";
import { IconAlertCircle, IconAlertTriangle, IconCheck, IconInfoCircle } from "@tabler/icons-react";

type InlineAlertTone = "error" | "info" | "success" | "warning";

type InlineAlertProps = AlertProps & {
  tone?: InlineAlertTone;
};

const toneConfig = {
  error: { color: "red", icon: <IconAlertCircle size={16} /> },
  info: { color: "cyan", icon: <IconInfoCircle size={16} /> },
  success: { color: "green", icon: <IconCheck size={16} /> },
  warning: { color: "yellow", icon: <IconAlertTriangle size={16} /> },
};

export const InlineAlert = ({ tone = "info", ...props }: InlineAlertProps) => {
  const config = toneConfig[tone];

  return <Alert color={config.color} icon={config.icon} radius="md" variant="light" {...props} />;
};
