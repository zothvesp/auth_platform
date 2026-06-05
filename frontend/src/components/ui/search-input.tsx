"use client";

import { TextInput, type TextInputProps } from "@mantine/core";
import { IconSearch, IconX } from "@tabler/icons-react";
import { AppIconButton } from "./icon-button";

type SearchInputProps = Omit<TextInputProps, "onChange"> & {
  onChange: (value: string) => void;
  value: string;
};

export const SearchInput = ({
  onChange,
  placeholder = "Search",
  value,
  ...props
}: SearchInputProps) => (
  <TextInput
    icon={<IconSearch size={15} />}
    placeholder={placeholder}
    value={value}
    onChange={(event) => onChange(event.currentTarget.value)}
    rightSection={
      value ? (
        <AppIconButton
          label="Clear search"
          icon={<IconX size={14} />}
          size="sm"
          onClick={() => onChange("")}
        />
      ) : null
    }
    size="xs"
    {...props}
  />
);
