import { ProtectedLayout } from "@components/layout/protected-layout";
import type { PropsWithChildren } from "react";

export default function SettingsLayout({ children }: PropsWithChildren) {
  return <ProtectedLayout>{children}</ProtectedLayout>;
}
