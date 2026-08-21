import { test } from "@wabou/test";

test("capture message scroller at HiDPI", async ({ page }) => {
  await page.getByRole("button", { name: "Message scroller" }).click();
  await page.getByRole("heading", { name: "Message scroller" }).waitFor();
  await page.getByRole("button", { name: "Append message" }).click();
  await page.getByRole("label", { name: "New event received." }).waitFor();
  await page.getByRole("button", { name: "Scroll to start" }).click();
  await page.getByRole("button", { name: "Scroll to end" }).waitFor();
});
