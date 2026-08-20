import { expect, test } from "@wabou/test";

test("toolbar exposes one tab stop and routes arrow keys", async ({ page }) => {
  await page.getByRole("button", { name: "Toolbar" }).click();

  const toolbar = page.getByRole("toolbar", { name: "Document formatting" });
  await toolbar.waitFor();
  const undo = page.getByRole("button", { name: "Undo" });
  const bold = page.getByRole("button", { name: "Bold" });

  await undo.click();
  await expect(undo).toBeFocused();
  await undo.press("ArrowRight");
  await expect(bold).toBeFocused();

  await bold.click();
  await expect(bold).toBePressed();
});
