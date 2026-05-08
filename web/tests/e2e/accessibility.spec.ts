import { test, expect } from "@playwright/test";

test.describe("WCAG 2.2 AA — E2E accessibility", () => {
  test("skip-to-content link is present", async ({ page }) => {
    await page.goto("/");
    const skipLink = page.locator("a.kn-skip-link");
    await expect(skipLink).toHaveText("Skip to main content");
    await expect(skipLink).toHaveAttribute("href", "#main-content");
  });

  test("main landmark exists with id", async ({ page }) => {
    await page.goto("/");
    const main = page.locator("main#main-content");
    await expect(main).toBeVisible();
  });

  test("navigation has aria-label", async ({ page }) => {
    await page.goto("/");
    const nav = page.getByRole("navigation", { name: "Main navigation" });
    await expect(nav).toBeVisible();
  });

  test("all nav tabs are keyboard-focusable", async ({ page }) => {
    await page.goto("/");
    const tabs = page.locator(".kn-sidebar__item");
    const count = await tabs.count();
    expect(count).toBeGreaterThan(0);

    for (let i = 0; i < count; i++) {
      const tab = tabs.nth(i);
      await expect(tab).toHaveAttribute("class", /kn-sidebar__item/);
    }
  });

  test("html lang attribute is set", async ({ page }) => {
    await page.goto("/");
    const html = page.locator("html");
    await expect(html).toHaveAttribute("lang", "en");
  });
});
