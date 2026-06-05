import { VerifyEmailView } from "@components/auth/verify-email-view";

type VerifyEmailPageProps = {
  searchParams: Promise<{
    token?: string;
  }>;
};

export default async function VerifyEmail({ searchParams }: VerifyEmailPageProps) {
  const params = await searchParams;
  return <VerifyEmailView token={params.token} />;
}
