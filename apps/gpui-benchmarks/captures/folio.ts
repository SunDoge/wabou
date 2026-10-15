import { test } from "@wabou/test";

test("capture Folio Desk", async ({ page }) => {
  await page.getByRole("button", { name: "Open Folio Desk" }).click();
  await page
    .getByRole("group", { name: "Folio Desk correspondence" })
    .waitFor();
});
