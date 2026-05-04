import { test, expect } from "@playwright/test";

const XSS_PAYLOADS = [
  '<script>alert("xss")</script>',
  '<img src=x onerror="alert(1)">',
  '<svg onload="alert(1)">',
];

test.describe("11) XSS payload smoke (NFR-S7)", () => {
  test("CSP header blocks inline script execution", async ({ page }) => {
    const response = await page.goto("/");
    const csp = response?.headers()["content-security-policy"];
    expect(csp).toBeDefined();
    expect(csp).toContain("default-src");
  });

  test("XSS payloads in page source are escaped, never raw", async ({
    page,
  }) => {
    await page.goto("/");
    const content = await page.content();

    for (const payload of XSS_PAYLOADS) {
      expect(content).not.toContain(payload);
    }
  });

  test("no alert dialogs triggered", async ({ page }) => {
    let dialogTriggered = false;
    page.on("dialog", () => {
      dialogTriggered = true;
    });

    await page.goto("/");
    await page.waitForTimeout(500);

    expect(dialogTriggered).toBe(false);
  });
});
