import { expect, test } from "@wabou/test";

test("progress exposes its animated value as native range semantics", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Progress" }).click();
  const progress = page.getByRole("progressbar", { name: "Build progress" });

  await expect(progress).toHaveValue("64 percent");
  await expect(progress).toHaveRange({ value: 64, min: 0, max: 100 });
  await page.getByRole("button", { name: "Advance" }).click();
  await expect(progress).toHaveValue("74 percent");
  await expect(progress).toHaveRange({ value: 74 });
  await page.getByRole("button", { name: "Reset" }).click();
  await expect(progress).toHaveValue("0 percent");
  await expect(progress).toHaveRange({ value: 0 });
});
