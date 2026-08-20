import { expect, test } from "@wabou/test";

test("progress publishes determinate and indeterminate native ranges", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Progress" }).click();

  const build = page.getByRole("progressbar", { name: "Build progress" });
  await expect(build).toHaveValue("64 percent");
  await expect(build).toHaveRange({ value: 64, min: 0, max: 100 });
  await page.getByRole("button", { name: "Advance" }).click();
  await expect(build).toHaveValue("74 percent");
  await expect(build).toHaveRange({ value: 74 });
  await page.getByRole("button", { name: "Reset" }).click();
  await expect(build).toHaveValue("0 percent");
  await expect(build).toHaveRange({ value: 0 });

  await expect(
    page.getByRole("progressbar", { name: "Downloaded release archive" }),
  ).toHaveRange({ value: 48, min: 0, max: 64 });

  const pending = await page
    .getByRole("progressbar", { name: "Resolving dependencies" })
    .snapshot();
  if (pending.numericValue !== null) {
    throw new Error(
      `indeterminate progress exposed numeric value ${pending.numericValue}`,
    );
  }
});
