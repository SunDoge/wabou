import { expect, test } from "@wabou/test";

test("shipped component roles remain native and actionable", async ({ page }) => {
  await page.getByRole("button", { name: "Tabs" }).click();
  await page
    .getByRole("tablist", { name: "Settings sections" })
    .waitFor();
  const account = page.getByRole("tab", { name: "Account" });
  const security = page.getByRole("tab", { name: "Security" });
  await expect(account).toBeSelected();
  await security.click();
  await expect(security).toBeSelected();
  await expect(account).toBeDeselected();

  await page.getByRole("button", { name: "Avatar" }).click();
  await page.getByRole("img", { name: "WA" }).waitFor();
});
