import { test } from "@wabou/test";

test("capture shared navigation viewport in the light theme", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Theme: Dark", index: 0 }).click();
  await page.getByRole("button", { name: "Navigation menu" }).click();
  await page.getByRole("heading", { name: "Navigation menu" }).waitFor();
  const products = page.getByRole("menuitem", { name: "Products" });
  await products.click();
  await page.getByRole("link", { name: "Wabou Runtime" }).waitFor();
  await products.press("ArrowRight");
  await page.getByRole("link", { name: "Guides" }).waitFor();
});
