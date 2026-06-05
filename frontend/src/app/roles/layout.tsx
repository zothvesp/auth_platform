import { ProtectedLayout } from "@components/layout/protected-layout";
import type { PropsWithChildren } from "react";

export default function RolesLayout({ children }: PropsWithChildren) {
  return <ProtectedLayout>{children}</ProtectedLayout>;
}
