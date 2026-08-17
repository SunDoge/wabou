import { expect, test } from "@wabou/test";

test("explicit locator index selects repeated controls without weakening strict mode", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Button" }).click();
  await expect(page.getByRole("button", { name: "Default" })).toHaveCount(2);
  const first = page.getByRole("button", { name: "Default", index: 0 });
  const second = page.getByRole("button", { name: "Default", index: 1 });
  const firstBounds = (await first.snapshot()).bounds;
  const secondBounds = (await second.snapshot()).bounds;
  expect(firstBounds.y === secondBounds.y).toBe(false);
  await second.click();
});
