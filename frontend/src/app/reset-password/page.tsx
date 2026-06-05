import { ResetPasswordForm } from "@components/auth/reset-password-form";
import { authProviderServer } from "@providers/auth-provider/auth-provider.server";
import { redirect } from "next/navigation";

type ResetPasswordPageProps = {
  searchParams: Promise<{
    token?: string;
  }>;
};

export default async function ResetPassword({ searchParams }: ResetPasswordPageProps) {
  const data = await authProviderServer.check();

  if (data.authenticated) {
    redirect(data.redirectTo || "/");
  }

  const params = await searchParams;
  return <ResetPasswordForm token={params.token} />;
}
