"use client";

import { createEmotionCache, MantineProvider } from "@mantine/core";
import { NotificationsProvider } from "@mantine/notifications";
import { authProviderClient } from "@providers/auth-provider/auth-provider.client";
import { dataProvider } from "@providers/data-provider";
import { Refine } from "@refinedev/core";
import { RefineKbar, RefineKbarProvider } from "@refinedev/kbar";
import routerProvider from "@refinedev/nextjs-router";
import { notificationProvider } from "@refinedev/mantine";
import React, { useEffect } from "react";
import { EmotionRegistry } from "./emotion-registry";
import { I18nProvider } from "@lib/i18n";

const SUPPRESSED_WARNINGS = [
  "Accessing element.ref was removed in React 19",
  "Invalid value for prop",
];

if (typeof window !== "undefined") {
  for (const method of ["error", "warn"] as const) {
    const original = console[method].bind(console);
    console[method] = (...args: unknown[]) => {
      const message = typeof args[0] === "string" ? args[0] : "";
      if (SUPPRESSED_WARNINGS.some((w) => message.includes(w))) return;
      original(...args);
    };
  }
}

type AppProvidersProps = {
  children: React.ReactNode;
};

export const AppProviders = ({ children }: AppProvidersProps) => {
  return (
    <EmotionRegistry>
      {(emotionCache) => (
        <ProviderContent emotionCache={emotionCache}>
          <I18nProvider>{children}</I18nProvider>
        </ProviderContent>
      )}
    </EmotionRegistry>
  );
};

type ProviderContentProps = AppProvidersProps & {
  emotionCache: ReturnType<typeof createEmotionCache>;
};

const ProviderContent = ({ children, emotionCache }: ProviderContentProps) => {
  return (
    <MantineProvider
      emotionCache={emotionCache}
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
            TextInput: {
              styles: (theme) => ({
                input: {
                  backgroundColor: theme.colors.dark[6],
                  borderColor: theme.colors.dark[4],
                  "&:focus": {
                    borderColor: theme.colors.cyan[6],
                  },
                },
              }),
            },
            PasswordInput: {
              styles: (theme) => ({
                input: {
                  backgroundColor: theme.colors.dark[6],
                  borderColor: theme.colors.dark[4],
                  "&:focus": {
                    borderColor: theme.colors.cyan[6],
                  },
                },
              }),
            },
            Select: {
              styles: (theme) => ({
                input: {
                  backgroundColor: theme.colors.dark[6],
                  borderColor: theme.colors.dark[4],
                  "&:focus": {
                    borderColor: theme.colors.cyan[6],
                  },
                },
              }),
            },
            Textarea: {
              styles: (theme) => ({
                input: {
                  backgroundColor: theme.colors.dark[6],
                  borderColor: theme.colors.dark[4],
                  "&:focus": {
                    borderColor: theme.colors.cyan[6],
                  },
                },
              }),
            },
            Badge: {
              styles: (theme) => ({
                root: {
                  fontWeight: 600,
                },
              }),
            },
            Tabs: {
              styles: (theme) => ({
                tab: {
                  "&[data-active]": {
                    color: theme.colors.cyan[4],
                    borderBottomColor: theme.colors.cyan[4],
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
                  create: "/users/create",
                  show: "/users/show/:id",
                  edit: "/users/edit/:id",
                  meta: {
                    canDelete: true,
                  },
                },
                {
                  name: "roles",
                  list: "/roles",
                  create: "/roles/create",
                  show: "/roles/show/:id",
                  edit: "/roles/edit/:id",
                  meta: {
                    canDelete: true,
                  },
                },
                {
                  name: "permissions",
                  list: "/permissions",
                  create: "/permissions/create",
                  show: "/permissions/show/:id",
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
                  edit: "/settings/edit/:id",
                  meta: {
                    canDelete: false,
                  },
                },
                {
                  name: "profile",
                  list: "/profile",
                  meta: {
                    label: "My profile",
                  },
                },
                {
                  name: "security",
                  list: "/security",
                  meta: {
                    label: "Security",
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
