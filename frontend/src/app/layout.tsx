import { Metadata } from "next";
import React from "react";

import { AppProviders } from "./providers";
import "../styles/global.css";

export const dynamic = "force-dynamic";

export const metadata: Metadata = {
  title: "OPSCTRL Auth Platform",
  description: "Authentication and authorization admin console",
  icons: {
    icon: "/favicon.ico",
  },
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        <AppProviders>{children}</AppProviders>
      </body>
    </html>
  );
}
