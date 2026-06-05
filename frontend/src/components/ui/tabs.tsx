"use client";

import { Tabs, type TabsProps } from "@mantine/core";

export const AppTabs = (props: TabsProps) => (
  <Tabs
    variant="outline"
    styles={(theme) => ({
      tab: {
        fontWeight: 700,
        fontSize: 12,
        "&[data-active]": {
          color: theme.colors.cyan[3],
        },
      },
    })}
    {...props}
  />
);
