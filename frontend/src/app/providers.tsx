"use client";

import { MantineProvider } from "@mantine/core";
import { NotificationsProvider } from "@mantine/notifications";
import { authProviderClient } from "@providers/auth-provider/auth-provider.client";
import { dataProvider } from "@providers/data-provider";
import { Refine } from "@refinedev/core";
import { RefineKbar, RefineKbarProvider } from "@refinedev/kbar";
import routerProvider from "@refinedev/nextjs-router";
import { notificationProvider } from "@refinedev/mantine";
import React from "react";

type AppProvidersProps = {
  children: React.ReactNode;
};

export const AppProviders = ({ children }: AppProvidersProps) => {
  return (
    <MantineProvider
      withGlobalStyles
      withNormalizeCSS
      theme={{
        colorScheme: "dark",
        globalStyles: (theme) => ({
          body: {
            backgroundColor: theme.colors.dark[8],
          },
        }),
        primaryColor: "cyan",
        fontFamily: "IBM Plex Sans, Inter, system-ui, sans-serif",
        fontFamilyMonospace: "JetBrains Mono, ui-monospace, SFMono-Regular, Menlo, monospace",
        headings: { fontFamily: "Fraunces, IBM Plex Sans, system-ui, sans-serif" },
        defaultRadius: "md",
        colors: {
          cyan: [
            "#e6fbff",
            "#b8f4ff",
            "#7cecfa",
            "#3be3f7",
            "#00d8ed",
            "#00b8cc",
            "#0097a8",
            "#007987",
            "#005d69",
            "#003e47",
          ],
          violet: [
            "#f5edff",
            "#e7d3ff",
            "#d0a8ff",
            "#b87bfa",
            "#a855f7",
            "#9333ea",
            "#7e22ce",
            "#6b21a8",
            "#581c87",
            "#3b0764",
          ],
        },
        shadows: {
          md: "var(--nsn-uif-refreshed-depth-popup)",
          xl: "var(--nsn-uif-refreshed-depth-modal)",
        },
        components: {
          Card: {
            styles: (theme) => ({
              root: {
                backgroundColor: theme.colors.dark[7],
                borderColor: "var(--nsn-uif-refreshed-border-subtle)",
              },
            }),
          },
          Table: {
            styles: (theme) => ({
              root: {
                "thead tr th": {
                  backgroundColor: theme.colors.dark[8],
                  color: theme.colors.gray[5],
                  fontSize: 11,
                  letterSpacing: 0.8,
                  textTransform: "uppercase",
                },
                "tbody tr:nth-of-type(even) td": {
                  backgroundColor: "rgba(255, 255, 255, 0.015)",
                },
              },
            }),
          },
        },
      }}
    >
      <NotificationsProvider position="bottom-right">
        <RefineKbarProvider>
          <Refine
            routerProvider={routerProvider}
            dataProvider={dataProvider}
            authProvider={authProviderClient}
            notificationProvider={notificationProvider}
            resources={[
              {
                name: "users",
                list: "/users",
                meta: {
                  canDelete: true,
                },
              },
              {
                name: "roles",
                list: "/roles",
                meta: {
                  canDelete: true,
                },
              },
              {
                name: "permissions",
                list: "/permissions",
              },
              {
                name: "audit_logs",
                list: "/audit-logs",
                meta: {
                  label: "Audit Logs",
                },
              },
              {
                name: "settings",
                list: "/settings",
                meta: {
                  canDelete: false,
                },
              },
            ]}
            options={{
              syncWithLocation: true,
              warnWhenUnsavedChanges: true,
              projectId: "HnwG15-dMguGL-ooxoZM",
            }}
          >
            {children}
            <RefineKbar />
          </Refine>
        </RefineKbarProvider>
      </NotificationsProvider>
    </MantineProvider>
  );
};
