import { test } from "@wabou/test";

test("indeterminate activity has task-specific native status names", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Spinner" }).click();
  await page.getByRole("status", { name: "Syncing workspace" }).waitFor();
  await page.getByRole("status", { name: "Saving workspace" }).waitFor();
});
