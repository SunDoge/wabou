import { expect, test } from "@wabou/test";

test("tree view expands, selects, and moves native focus", async ({ page }) => {
  await page.getByRole("button", { name: "Tree view" }).click();

  const workspace = page.getByRole("treeitem", { name: "wabou" });
  await expect(workspace).toBeExpanded();

  const packages = page.getByRole("treeitem", { name: "packages" });
  await expect(packages).toBeExpanded();
  await packages.press("ArrowRight");
  await expect(page.getByRole("treeitem", { name: "core" })).toBeFocused();

  const apps = page.getByRole("treeitem", { name: "apps" });
  await apps.click();
  await expect(apps).toBeExpanded();
  const gallery = page.getByRole("treeitem", { name: "gallery" });
  await gallery.click();
  await expect(gallery).toBeSelected();
  await expect(
    page.getByRole("status", { name: "Selected tree node" }),
  ).toHaveText("gallery");
});
