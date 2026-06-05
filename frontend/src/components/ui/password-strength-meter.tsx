"use client";

import { Progress, Stack, Text } from "@mantine/core";

type PasswordStrengthMeterProps = {
  value: string;
};

const checks = [
  { label: "8+ chars", test: (value: string) => value.length >= 8 },
  { label: "Uppercase", test: (value: string) => /[A-Z]/.test(value) },
  { label: "Lowercase", test: (value: string) => /[a-z]/.test(value) },
  { label: "Number", test: (value: string) => /\d/.test(value) },
  { label: "Symbol", test: (value: string) => /[^A-Za-z0-9]/.test(value) },
];

export const PasswordStrengthMeter = ({ value }: PasswordStrengthMeterProps) => {
  const passed = checks.filter((check) => check.test(value));
  const score = (passed.length / checks.length) * 100;
  const color = score >= 80 ? "green" : score >= 50 ? "yellow" : "red";

  return (
    <Stack spacing={5}>
      <Progress value={score} color={color} size="sm" radius="xl" />
      <Text size="xs" color="dimmed">
        {passed.length}/{checks.length} checks passed ·{" "}
        {checks
          .filter((check) => !check.test(value))
          .map((check) => check.label)
          .join(", ") || "Strong password"}
      </Text>
    </Stack>
  );
};
