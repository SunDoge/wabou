import { expect, test } from "@wabou/test";

test("selection controls expose semantic interaction state", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Checkbox" }).click();

  const terms = page.getByRole("checkbox", {
    name: "Accept the terms and conditions",
  });
  await expect(terms).toBeUnchecked();
  await terms.click();
  await expect(terms).toBeChecked();
  await expect(
    page.getByRole("checkbox", { name: "Selected by default" }),
  ).toBeChecked();
  await expect(
    page.getByRole("checkbox", { name: "Some child items selected" }),
  ).toBeIndeterminate();

  await page.getByRole("button", { name: "Toggle" }).click();
  await expect(page.getByRole("button", { name: "Toggle bold" })).toBePressed();
  await expect(
    page.getByRole("button", { name: "Toggle italic" }),
  ).toBeUnpressed();
});
