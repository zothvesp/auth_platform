import { RegisterForm } from "@components/auth/register-form";
import { authProviderServer } from "@providers/auth-provider/auth-provider.server";
import { redirect } from "next/navigation";

export default async function Register() {
  const data = await authProviderServer.check();

  if (data.authenticated) {
    redirect(data.redirectTo || "/");
  }

  return <RegisterForm />;
}
