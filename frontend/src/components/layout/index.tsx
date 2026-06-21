"use client";

import { AppShell, Box, Header, MediaQuery, Text } from "@mantine/core";
import type { PropsWithChildren } from "react";
import { Breadcrumb } from "../breadcrumb";
import { Menu } from "../menu";
import { HeaderContent } from "./header";

export const Layout: React.FC<PropsWithChildren> = ({ children }) => {
  return (
    <AppShell
      padding={0}
      navbar={<Menu />}
      header={
        <Header height={56} px="lg">
          <Box h="100%" sx={{ display: "flex", alignItems: "center", gap: 16 }}>
            <MediaQuery smallerThan="sm" styles={{ display: "none" }}>
              <Text size="sm" color="dimmed" weight={600} mr="xl">
                Auth Platform
              </Text>
            </MediaQuery>
            <Breadcrumb />
            <Box sx={{ flex: 1 }} />
            <HeaderContent />
          </Box>
        </Header>
      }
      styles={(theme) => ({
        main: {
          minHeight: "100vh",
          background: theme.colors.dark[8],
        },
      })}
    >
      <Box p="lg">{children}</Box>
    </AppShell>
  );
};
