"use client";

import { CacheProvider } from "@emotion/react";
import { createEmotionCache } from "@mantine/core";
import { useServerInsertedHTML } from "next/navigation";
import { useState, type ReactNode } from "react";

type EmotionRegistryProps = {
  children: (cache: ReturnType<typeof createEmotionCache>) => ReactNode;
};

export function EmotionRegistry({ children }: EmotionRegistryProps) {
  const [{ cache, flush }] = useState(() => {
    const cache = createEmotionCache({ key: "mantine" });
    cache.compat = true;

    const originalInsert = cache.insert;
    let inserted: string[] = [];

    cache.insert = (...args) => {
      const serialized = args[1];

      if (cache.inserted[serialized.name] === undefined) {
        inserted.push(serialized.name);
      }

      return originalInsert(...args);
    };

    const flush = () => {
      const names = inserted;
      inserted = [];
      return names;
    };

    return { cache, flush };
  });

  useServerInsertedHTML(() => {
    const names = flush();

    if (names.length === 0) {
      return null;
    }

    const styles = names
      .map((name) => cache.inserted[name])
      .filter((style): style is string => typeof style === "string")
      .join("");

    return (
      <style
        data-emotion={`${cache.key} ${names.join(" ")}`}
        dangerouslySetInnerHTML={{ __html: styles }}
      />
    );
  });

  return <CacheProvider value={cache}>{children(cache)}</CacheProvider>;
}
