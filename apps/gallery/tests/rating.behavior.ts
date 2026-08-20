import { expect, test } from "@wabou/test";

test("rating exposes native radio semantics and keyboard selection", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Rating" }).click();
  // The page also demonstrates read-only and disabled groups. Select the first
  // authored occurrence until native locators gain component-subtree scoping.
  const fourth = page.getByRole("radio", { name: "4 stars", index: 0 });

  await fourth.click();
  await expect(fourth).toBeChecked();
  await expect(
    page.getByRole("status", { name: "Framework rating value" }),
  ).toHaveText("4 of 5 stars");

  await fourth.press("ArrowRight");
  await expect(
    page.getByRole("radio", { name: "5 stars", index: 0 }),
  ).toBeChecked();
  await expect(
    page.getByRole("status", { name: "Framework rating value" }),
  ).toHaveText("5 of 5 stars");
});
