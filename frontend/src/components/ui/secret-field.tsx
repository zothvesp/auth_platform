"use client";

import { Group, PasswordInput, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconEye, IconEyeOff } from "@tabler/icons-react";
import { AppIconButton } from "./icon-button";
import { CopyValueButton } from "./copy-button";

type SecretFieldProps = {
  label?: string;
  value: string;
};

export const SecretField = ({ label = "Secret", value }: SecretFieldProps) => {
  const [visible, handlers] = useDisclosure(false);

  return (
    <Group spacing="xs" align="flex-end" noWrap>
      {visible ? (
        <TextInput label={label} value={value} readOnly sx={{ flex: 1 }} />
      ) : (
        <PasswordInput label={label} value={value} readOnly sx={{ flex: 1 }} />
      )}
      <AppIconButton
        label={visible ? "Hide secret" : "Reveal secret"}
        icon={visible ? <IconEyeOff size={16} /> : <IconEye size={16} />}
        onClick={handlers.toggle}
      />
      <CopyValueButton value={value} />
    </Group>
  );
};
