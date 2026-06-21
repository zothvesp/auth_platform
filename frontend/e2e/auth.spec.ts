import { test, expect } from "@playwright/test";

test("login page loads", async ({ page }) => {
  const response = await page.goto("/login");
  expect(response?.status()).toBe(200);
  await page.waitForLoadState("networkidle");
  await expect(page.locator("body")).toBeVisible();
});

test("login form has email and password fields", async ({ page }) => {
  await page.goto("/login");
  await page.waitForLoadState("networkidle");
  const emailInput = page.locator('input[type="email"], input[name="email"]').first();
  const passwordInput = page.locator('input[type="password"]').first();
  await expect(emailInput).toBeVisible({ timeout: 10000 });
  await expect(passwordInput).toBeVisible({ timeout: 10000 });
});

test("register page loads", async ({ page }) => {
  const response = await page.goto("/register");
  expect(response?.status()).toBe(200);
  await page.waitForLoadState("networkidle");
  await expect(page.locator("body")).toBeVisible();
});
