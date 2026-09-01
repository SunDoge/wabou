import { expect, test } from "@wabou/test";

test("sheet remains mounted for its native exit transition", async ({ page }) => {
  await page.getByRole("button", { name: "Sheet" }).click();
  await page.getByRole("button", { name: "Open sheet" }).click();

  const sheet = page.getByRole("dialog", { name: "Edit profile" });
  await sheet.waitFor();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(sheet).toBeAbsent();
  await page.getByRole("button", { name: "Open sheet" }).waitFor();
});
