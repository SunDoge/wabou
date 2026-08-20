import { expect, test } from "@wabou/test";

test("gallery layout control enables the native diagnostic layer", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Overview" }).click();
  const overlay = page.getByRole("switch", { name: "Layout overlay" });
  await expect(overlay).toBeUnchecked();
  await overlay.click();
  await expect(overlay).toBeChecked();
});
