import { MfaForm } from "@components/auth/mfa-form";
import { authProviderServer } from "@providers/auth-provider/auth-provider.server";
import { redirect } from "next/navigation";

export default async function MfaPage() {
  const data = await authProviderServer.check();

  if (data.authenticated) {
    redirect(data.redirectTo || "/");
  }

  return <MfaForm />;
}
