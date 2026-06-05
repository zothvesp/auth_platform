"use client";

import {
  Checkbox,
  NativeSelect,
  PasswordInput,
  Select,
  Switch,
  TextInput,
  Textarea,
  type CheckboxProps,
  type NativeSelectProps,
  type PasswordInputProps,
  type SelectProps,
  type SwitchProps,
  type TextInputProps,
  type TextareaProps,
} from "@mantine/core";

const fieldStyles = {
  label: {
    fontSize: 11,
    fontWeight: 700,
    letterSpacing: 0.7,
    textTransform: "uppercase" as const,
  },
  input: {
    minHeight: 36,
  },
};

export const AppTextInput = (props: TextInputProps) => (
  <TextInput radius="md" size="sm" styles={fieldStyles} {...props} />
);

export const AppPasswordInput = (props: PasswordInputProps) => (
  <PasswordInput radius="md" size="sm" styles={fieldStyles} {...props} />
);

export const AppTextarea = (props: TextareaProps) => (
  <Textarea radius="md" size="sm" minRows={4} styles={fieldStyles} {...props} />
);

export const AppSelect = (props: SelectProps) => (
  <Select radius="md" size="sm" searchable clearable styles={fieldStyles} {...props} />
);

export const AppNativeSelect = (props: NativeSelectProps) => (
  <NativeSelect radius="md" size="sm" styles={fieldStyles} {...props} />
);

export const AppCheckbox = (props: CheckboxProps) => (
  <Checkbox
    color="cyan"
    radius="sm"
    styles={(theme) => ({
      input: {
        cursor: props.disabled ? "default" : "pointer",
        "&:focus": {
          boxShadow: `0 0 0 2px ${theme.fn.rgba(theme.colors.cyan[4], 0.55)}`,
        },
      },
    })}
    {...props}
  />
);

export const AppSwitch = (props: SwitchProps) => (
  <Switch
    color="cyan"
    radius="xl"
    styles={(theme) => ({
      input: {
        cursor: props.disabled ? "default" : "pointer",
        "&:focus": {
          boxShadow: `0 0 0 2px ${theme.fn.rgba(theme.colors.cyan[4], 0.55)}`,
        },
      },
    })}
    {...props}
  />
);
