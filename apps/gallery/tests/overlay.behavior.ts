import { expect, test } from "@wabou/test";

test("floating notifications yield semantics to a modal and recover", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Overlay" }).click();
  await page.getByRole("button", { name: "Show notification" }).click();

  const notification = page.getByRole("status", {
    name: "Vault synchronized",
  });
  await notification.waitFor();

  await page.getByRole("button", { name: "Open modal overlay" }).click();
  const dialog = page.getByRole("dialog", { name: "Overlay settings" });
  await dialog.waitFor();
  await expect(notification).toBeAbsent();

  await page.getByRole("button", { name: "Close" }).click();
  await expect(dialog).toBeAbsent();
  await notification.waitFor();

  await page.getByRole("button", { name: "Dismiss" }).click();
  await expect(notification).toBeAbsent();
});
