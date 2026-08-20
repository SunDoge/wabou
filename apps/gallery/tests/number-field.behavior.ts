import { expect, test } from "@wabou/test";

test("number field exposes native stepping semantics", async ({ page }) => {
  await page.getByRole("button", { name: "Number field" }).click();
  const field = page.getByRole("spinbutton", {
    name: "Download concurrency",
  });

  await expect(field).toHaveRange({ value: 4, min: 1, max: 32 });
  await field.press("ArrowUp");
  await expect(field).toHaveRange({ value: 5, min: 1, max: 32 });
  await expect(
    page.getByRole("status", { name: "Download concurrency value" }),
  ).toHaveText("5 concurrent tasks");
});
