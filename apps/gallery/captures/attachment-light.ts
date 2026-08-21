import { test } from "@wabou/test";

test("capture attachment states in the light theme", async ({ page }) => {
  await page.getByRole("button", { name: "Theme: Dark", index: 0 }).click();
  await page.getByRole("button", { name: "Attachment" }).click();
  await page.getByRole("heading", { name: "Attachment" }).waitFor();
  await page.getByRole("button", { name: "Retry recording upload" }).waitFor();
});
