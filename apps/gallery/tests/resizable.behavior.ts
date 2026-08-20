import { expect, test } from "@wabou/test";

test("resizable panels support native pointer and keyboard input", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Resizable" }).click();
  const handle = page.getByRole("separator", {
    name: "Resize navigation panel",
  });

  await expect(handle).toHaveRange({ value: 32, min: 20, max: 60 });
  await handle.dragBy(300, 0);
  await expect(page.getByRole("status", { name: "Panel sizes" })).toHaveText(
    "60% navigation · 40% content",
  );
  await handle.press("Home");
  await expect(handle).toHaveRange({ value: 20, min: 20, max: 60 });
});
