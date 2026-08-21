import { test } from "@wabou/test";

test("capture carousel at HiDPI", async ({ page }) => {
  await page.getByRole("button", { name: "Carousel" }).click();
  await page.getByRole("heading", { name: "Carousel" }).waitFor();
  await page.getByRole("button", { name: "Next slide" }).click();
  await page.getByRole("status", { name: "Slide 2 of 3" }).waitFor();
});
