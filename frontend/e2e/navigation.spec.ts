import { test, expect } from "@playwright/test";

test("root page responds", async ({ page }) => {
  const response = await page.goto("/");
  expect(response?.status()).toBe(200);
});

test("403 page responds", async ({ page }) => {
  const response = await page.goto("/forbidden");
  expect(response?.status()).toBe(200);
});
