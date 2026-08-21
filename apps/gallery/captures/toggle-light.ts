import { test } from "@wabou/test";

test("capture single and multiple toggle groups", async ({ page }) => {
  await page.getByRole("button", { name: "Theme: Dark", index: 0 }).click();
  await page.getByRole("button", { name: "Toggle" }).click();
  await page.getByRole("heading", { name: "Toggle" }).waitFor();

  await page.getByRole("button", { name: "Global" }).click();
  await page.getByRole("status", { name: "Routing mode: global" }).waitFor();

  const bold = page.getByRole("button", { name: "Bold" });
  await bold.press("ArrowRight");
  await page.getByRole("button", { name: "Italic" }).press("Enter");
  await page.getByRole("button", { name: "Underline" }).click();
  await page
    .getByRole("status", { name: "Formatting: Bold, Italic, Underline" })
    .waitFor();
});
