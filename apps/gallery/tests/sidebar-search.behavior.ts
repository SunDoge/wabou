import { expect, test } from "@wabou/test";

test("sidebar search filters navigation and clears through native input", async ({
  page,
}) => {
  const search = page.getByRole("textbox", { name: "Search components" });
  await search.type("tree");
  await page.getByRole("button", { name: "Tree view" }).waitFor();
  await expect(page.getByRole("button", { name: "Button" })).toBeAbsent();

  await search.press("Escape");
  await expect(search).toBeFocused();
  await page.getByRole("button", { name: "Overview" }).waitFor();
});
