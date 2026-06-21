import { describe, it, expect } from "vitest";
import { renderHook } from "@testing-library/react";
import { type ReactNode } from "react";
import { I18nProvider, useI18n, useTranslations } from "./i18n";
import en from "./translations";

const wrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider>{children}</I18nProvider>
);

describe("useTranslations", () => {
  it("returns the full translations object", () => {
    const { result } = renderHook(() => useTranslations(), { wrapper });
    expect(result.current).toEqual(en);
  });

  it("has common keys", () => {
    const { result } = renderHook(() => useTranslations(), { wrapper });
    expect(result.current.common.cancel).toBe("Cancel");
    expect(result.current.auth.signIn).toBe("Sign in");
  });
});

describe("useI18n", () => {
  it("returns locale and setLocale", () => {
    const { result } = renderHook(() => useI18n(), { wrapper });
    expect(result.current.locale).toBe("en");
    expect(typeof result.current.setLocale).toBe("function");
  });

  it("returns t as translations object", () => {
    const { result } = renderHook(() => useI18n(), { wrapper });
    expect(result.current.t).toEqual(en);
  });
});
