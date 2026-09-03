import { expect, test } from "@wabou/test";

test("captures responsive starter prompts", async ({ page }) => {
  await expect(
    page.getByRole("group", { name: "Starter prompts" }),
  ).toHaveCount(1);
  await expect(
    page.getByRole("button", { name: "Review current changes" }),
  ).toHaveCount(1);
  await expect(
    page.getByRole("button", { name: "Run project checks" }),
  ).toHaveCount(1);
  await expect(
    page.getByRole("button", { name: "Plan a feature" }),
  ).toHaveCount(1);
  await expect(
    page.getByRole("button", { name: "Scroll to end" }),
  ).toHaveCount(0);
});
