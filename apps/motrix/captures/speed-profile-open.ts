import { test } from "@wabou/test";

test("retain an open speed-profile relationship for capture", async ({
  page,
}) => {
  await page.getByRole("combobox", { name: "Speed profile" }).click();
  const listbox = page.getByRole("listbox", { name: "Speed profile" });
  await listbox.waitFor();
  await listbox.press("ArrowDown");
});
