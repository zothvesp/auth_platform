"use client";

import { Anchor, Breadcrumbs, Text } from "@mantine/core";
import { useBreadcrumb } from "@refinedev/core";
import Link from "next/link";

export const Breadcrumb = () => {
  const { breadcrumbs } = useBreadcrumb();

  return (
    <Breadcrumbs separator="/" styles={{ separator: { marginInline: 8 } }}>
      {breadcrumbs.map((breadcrumb) => {
        return (
          <span key={`breadcrumb-${breadcrumb.label}`}>
            {breadcrumb.href ? (
              <Anchor component={Link} href={breadcrumb.href} size="sm">
                {breadcrumb.label}
              </Anchor>
            ) : (
              <Text component="span" size="sm" color="dimmed">
                {breadcrumb.label}
              </Text>
            )}
          </span>
        );
      })}
    </Breadcrumbs>
  );
};
