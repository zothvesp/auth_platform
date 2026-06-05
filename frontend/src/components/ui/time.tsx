"use client";

import { Text, Tooltip } from "@mantine/core";

type TimeTextProps = {
  value?: Date | string | null;
};

const toDate = (value?: Date | string | null) => {
  if (!value) return null;
  const date = value instanceof Date ? value : new Date(value);
  return Number.isNaN(date.getTime()) ? null : date;
};

export const DateTimeText = ({ value }: TimeTextProps) => {
  const date = toDate(value);
  if (!date) return <Text size="sm">-</Text>;

  return (
    <Tooltip label={date.toISOString()}>
      <Text size="sm">
        {date.toLocaleString(undefined, {
          dateStyle: "medium",
          timeStyle: "short",
        })}
      </Text>
    </Tooltip>
  );
};

export const RelativeTimeText = ({ value }: TimeTextProps) => {
  const date = toDate(value);
  if (!date) return <Text size="sm">-</Text>;

  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  const abs = Math.abs(seconds);
  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 31536000],
    ["month", 2592000],
    ["week", 604800],
    ["day", 86400],
    ["hour", 3600],
    ["minute", 60],
  ];
  const [unit, size] = units.find(([, size]) => abs >= size) ?? ["second", 1];
  const amount = Math.round(seconds / size);
  const formatter = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });

  return (
    <Tooltip label={<DateTimeText value={date} />}>
      <Text size="sm">{formatter.format(-amount, unit)}</Text>
    </Tooltip>
  );
};
