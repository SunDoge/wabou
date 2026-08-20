import { test } from "@wabou/test";

test("show the empty search result state for capture", async ({ page }) => {
  await page.getByRole("button", { name: "Downloads" }).click();
  await page
    .getByRole("textbox", { name: "Search downloads" })
    .type("no-such-download");
  await page.getByRole("button", { name: "Clear search" }).waitFor();
});
