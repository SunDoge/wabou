import { expect, test } from "@wabou/test";

test("rating exposes native radio semantics and keyboard selection", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Rating" }).click();
  const rating = page.getByRole("radiogroup", { name: "Framework rating" });
  const fourth = rating.getByRole("radio", { name: "4 stars" });

  await fourth.click();
  await expect(fourth).toBeChecked();
  await expect(
    page.getByRole("status", { name: "Framework rating value" }),
  ).toHaveText("4 of 5 stars");

  await fourth.press("ArrowRight");
  await expect(rating.getByRole("radio", { name: "5 stars" })).toBeChecked();
  await expect(
    page.getByRole("status", { name: "Framework rating value" }),
  ).toHaveText("5 of 5 stars");
});
