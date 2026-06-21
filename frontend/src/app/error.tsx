"use client";

import { AppButton, ErrorState } from "@components/ui";
import { useTranslations } from "@lib/i18n";

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  const t = useTranslations();

  return (
    <ErrorState
      title={error.name || t.auth.signInFailed}
      message={error.message || "An unexpected error occurred."}
      action={
        <AppButton appVariant="secondary" onClick={reset}>
          {t.common.back}
        </AppButton>
      }
    />
  );
}
