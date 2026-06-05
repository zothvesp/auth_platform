"use client";

import { CopyButton as MantineCopyButton } from "@mantine/core";
import { IconCheck, IconCopy } from "@tabler/icons-react";
import { AppButton } from "./button";

type CopyValueButtonProps = {
  label?: string;
  value: string;
};

export const CopyValueButton = ({ label = "Copy", value }: CopyValueButtonProps) => (
  <MantineCopyButton value={value} timeout={1600}>
    {({ copied, copy }) => (
      <AppButton
        appVariant="secondary"
        icon={copied ? <IconCheck size={15} /> : <IconCopy size={15} />}
        onClick={copy}
      >
        {copied ? "Copied" : label}
      </AppButton>
    )}
  </MantineCopyButton>
);
