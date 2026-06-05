"use client";

import { Box, Group, ScrollArea, Text } from "@mantine/core";
import { CopyValueButton } from "./copy-button";

type CodeBlockProps = {
  copyable?: boolean;
  language?: string;
  value: string;
};

export const CodeBlock = ({ copyable = true, language, value }: CodeBlockProps) => (
  <Box
    sx={(theme) => ({
      background: theme.colors.dark[8],
      border: `1px solid ${theme.colors.dark[4]}`,
      borderRadius: theme.radius.md,
      overflow: "hidden",
    })}
  >
    <Group position="apart" px="sm" py={6} sx={(theme) => ({ borderBottom: `1px solid ${theme.colors.dark[4]}` })}>
      <Text size="xs" color="dimmed" transform="uppercase" weight={700}>
        {language ?? "Code"}
      </Text>
      {copyable ? <CopyValueButton label="Copy" value={value} /> : null}
    </Group>
    <ScrollArea>
      <Box component="pre" m={0} p="sm" sx={{ fontSize: 12, lineHeight: 1.6 }}>
        <code>{value}</code>
      </Box>
    </ScrollArea>
  </Box>
);
