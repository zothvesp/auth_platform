import { describe, it, expect } from "vitest";
import en from "./translations";

describe("translations", () => {
  it("has top-level sections", () => {
    const sections = ["common", "auth", "pages", "authShell", "forms", "table", "detail", "confirm"] as const;
    for (const key of sections) {
      expect(en).toHaveProperty(key);
    }
  });

  it("common has basic keys", () => {
    expect(en.common.cancel).toBe("Cancel");
    expect(en.common.save).toBe("Save");
    expect(en.common.loading).toBe("Loading");
  });

  it("auth has sign-in text", () => {
    expect(en.auth.signIn).toBe("Sign in");
    expect(typeof en.auth.signIn).toBe("string");
  });

  it("pages has expected sub-sections", () => {
    expect(en.pages.users).toBeDefined();
    expect(en.pages.roles).toBeDefined();
    expect(en.pages.permissions).toBeDefined();
    expect(en.pages.auditLogs).toBeDefined();
    expect(en.pages.settings).toBeDefined();
  });

  it("forms has common field labels", () => {
    expect(en.forms.email).toBe("Email");
    expect(en.forms.password).toBe("Password");
  });

  it("contains only strings or functions", () => {
    const flatten = (obj: Record<string, unknown>): unknown[] =>
      Object.values(obj).flatMap((v) => (typeof v === "object" && v !== null && !Array.isArray(v) ? flatten(v as Record<string, unknown>) : [v]));
    const values = flatten(en as unknown as Record<string, unknown>);
    for (const v of values) {
      expect(typeof v === "string" || typeof v === "function" || Array.isArray(v)).toBe(true);
    }
  });
});
