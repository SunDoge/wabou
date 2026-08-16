import { test } from "@wabou/test";

test("modal descendants remain available to semantic locators", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Dialog" }).click();
  await page.getByRole("button", { name: "Delete project" }).click();
  await page.getByRole("dialog", { name: "Delete project" }).waitFor();
  await page.getByRole("button", { name: "Delete" }).click();
  await page.getByRole("button", { name: "Delete project" }).waitFor();
});
