import { test } from "@wabou/test";

test("capture aspect ratio constraints at HiDPI", async ({ page }) => {
  await page.getByRole("button", { name: "Aspect ratio" }).click();
  await page.getByRole("heading", { name: "Aspect ratio" }).waitFor();
});
