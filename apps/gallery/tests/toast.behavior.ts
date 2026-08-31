import { expect, test } from "@wabou/test";

test("toast enters the floating region and remains interactive", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Toast" }).click();
  await page.getByRole("button", { name: "Success toast" }).click();

  const toast = page.getByRole("status", { name: "Project saved" });
  await toast.waitFor();
  await page.getByRole("button", { name: "Dismiss Project saved" }).click();
  await expect(toast).toBeAbsent();
});
