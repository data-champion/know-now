import { test, expect } from "@playwright/test";

test.describe("Dashboard E2E", () => {
  test.describe("1) Launch + session bootstrap", () => {
    test("navigating to root without session returns 401 on API", async ({
      page,
    }) => {
      const response = await page.request.get(
        "http://127.0.0.1:3827/api/v1/version",
      );
      expect(response.status()).toBe(401);
    });
  });

  test.describe("2) Entity browsing", () => {
    test("entity list renders on the main page", async ({ page }) => {
      await page.goto("/");
      const entityTab = page.getByRole("button", { name: "Entities" });
      await expect(entityTab).toBeVisible();
    });
  });

  test.describe("3) Relationship graph + table fallback", () => {
    test("relationships tab exists and is clickable", async ({ page }) => {
      await page.goto("/");
      const relTab = page.getByRole("button", { name: "Relationships" });
      await expect(relTab).toBeVisible();
      await relTab.click();
    });
  });

  test.describe("4) Generation status", () => {
    test("generation tab is accessible", async ({ page }) => {
      await page.goto("/");
      const genTab = page.getByRole("button", { name: "Generation" });
      await expect(genTab).toBeVisible();
      await genTab.click();
    });
  });

  test.describe("5) Docs viewer", () => {
    test("docs tab is accessible", async ({ page }) => {
      await page.goto("/");
      const docsTab = page.getByRole("button", { name: "Docs" });
      await expect(docsTab).toBeVisible();
      await docsTab.click();
    });
  });

  test.describe("6) Manifest viewer", () => {
    test("manifest tab is accessible", async ({ page }) => {
      await page.goto("/");
      const manifestTab = page.getByRole("button", { name: "Manifest" });
      await expect(manifestTab).toBeVisible();
      await manifestTab.click();
    });
  });

  test.describe("7) Traceability", () => {
    test("traceability tab is accessible", async ({ page }) => {
      await page.goto("/");
      const traceTab = page.getByRole("button", { name: "Traceability" });
      await expect(traceTab).toBeVisible();
      await traceTab.click();
    });
  });

  test.describe("8) Health/admin", () => {
    test("health tab is accessible", async ({ page }) => {
      await page.goto("/");
      const healthTab = page.getByRole("button", { name: "Health" });
      await expect(healthTab).toBeVisible();
      await healthTab.click();
    });
  });

  test.describe("9) Review", () => {
    test("review tab is accessible", async ({ page }) => {
      await page.goto("/");
      const reviewTab = page.getByRole("button", { name: "Review" });
      await expect(reviewTab).toBeVisible();
      await reviewTab.click();
    });
  });
});
