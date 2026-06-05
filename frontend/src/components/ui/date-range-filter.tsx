"use client";

import { Group } from "@mantine/core";
import { AppTextInput } from "./form";

export type DateRangeValue = {
  from?: string;
  to?: string;
};

type DateRangeFilterProps = {
  label?: string;
  onChange: (value: DateRangeValue) => void;
  value: DateRangeValue;
};

export const DateRangeFilter = ({ label = "Date range", onChange, value }: DateRangeFilterProps) => (
  <Group spacing="xs" noWrap>
    <AppTextInput
      aria-label={`${label} from`}
      label="From"
      type="date"
      value={value.from ?? ""}
      onChange={(event) => onChange({ ...value, from: event.currentTarget.value })}
    />
    <AppTextInput
      aria-label={`${label} to`}
      label="To"
      type="date"
      value={value.to ?? ""}
      onChange={(event) => onChange({ ...value, to: event.currentTarget.value })}
    />
  </Group>
);
