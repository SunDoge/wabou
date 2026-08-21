import { test } from "@wabou/test";

test("capture aspect ratio constraints in the light theme", async ({ page }) => {
  await page.getByRole("button", { name: "Theme: Dark", index: 0 }).click();
  await page.getByRole("button", { name: "Aspect ratio" }).click();
  await page.getByRole("heading", { name: "Aspect ratio" }).waitFor();
});
