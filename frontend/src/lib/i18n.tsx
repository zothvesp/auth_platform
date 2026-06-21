"use client";

import { createContext, useCallback, useContext, useMemo, useState } from "react";
import en, { type Translations } from "./translations";

type Locale = "en";

const locales: Record<Locale, Translations> = { en };

type I18nContextValue = {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: Translations;
};

const I18nContext = createContext<I18nContextValue>({
  locale: "en",
  setLocale: () => {},
  t: en,
});

export const I18nProvider = ({ children }: { children: React.ReactNode }) => {
  const [locale, setLocale] = useState<Locale>("en");

  const value = useMemo(
    () => ({
      locale,
      setLocale,
      t: locales[locale],
    }),
    [locale],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
};

export const useI18n = () => useContext(I18nContext);

/** Shorthand: returns the translations object for the current locale. */
export const useTranslations = () => useI18n().t;
