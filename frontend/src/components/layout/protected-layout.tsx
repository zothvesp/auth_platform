import { Layout as BaseLayout } from "@components/layout";
import { authProviderServer } from "@providers/auth-provider/auth-provider.server";
import { redirect } from "next/navigation";
import type { PropsWithChildren } from "react";

export async function ProtectedLayout({ children }: PropsWithChildren) {
  const data = await authProviderServer.check();

  if (!data.authenticated) {
    redirect(data.redirectTo || "/login");
  }

  return <BaseLayout>{children}</BaseLayout>;
}
