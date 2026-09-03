import { expect, test } from "@wabou/test";

test("drawer completes its native exit without another user input", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Drawer" }).click();
  await page.getByRole("button", { name: "Open drawer" }).click();

  const drawer = page.getByRole("dialog", { name: "Create task" });
  await drawer.waitFor();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(drawer).toBeAbsent();
  await page.getByRole("button", { name: "Open drawer" }).waitFor();
});
