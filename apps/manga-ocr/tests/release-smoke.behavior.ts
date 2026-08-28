import { test } from "@wabou/test";

test("packaged Manga OCR boots the starter workspace", async ({ page }) => {
  await page.getByRole("button", { name: "Open manga pages" }).waitFor();
  await page.getByRole("button", { name: "Open manga directory" }).waitFor();
});
