"use client";

import { Badge, Tabs } from "@mantine/core";
import type { ReactNode } from "react";
import { AppTabs } from "./tabs";

export type PageTab = {
  count?: number;
  icon?: ReactNode;
  label: string;
  value: string;
};

type PageTabsProps = {
  onChange: (value: string) => void;
  tabs: PageTab[];
  value: string;
};

export const PageTabs = ({ onChange, tabs, value }: PageTabsProps) => (
  <AppTabs value={value} onTabChange={onChange}>
    <Tabs.List>
      {tabs.map((tab) => (
        <Tabs.Tab key={tab.value} value={tab.value} icon={tab.icon}>
          {tab.label}
          {typeof tab.count === "number" ? (
            <Badge ml={6} size="xs" variant="light">
              {tab.count}
            </Badge>
          ) : null}
        </Tabs.Tab>
      ))}
    </Tabs.List>
  </AppTabs>
);
