import { ForgotPasswordForm } from "@components/auth/forgot-password-form";
import { authProviderServer } from "@providers/auth-provider/auth-provider.server";
import { redirect } from "next/navigation";

export default async function ForgotPassword() {
  const data = await authProviderServer.check();

  if (data.authenticated) {
    redirect(data.redirectTo || "/");
  }

  return <ForgotPasswordForm />;
}
