import { expect, test } from "@wabou/test";

test("select exposes one combobox and listbox semantic model", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Select" }).click();
  const combobox = page.getByRole("combobox", { name: "UI framework" });
  await combobox.click();
  await expect(combobox).toBeExpanded();
  await page.getByRole("listbox", { name: "UI framework" }).waitFor();
  await expect(
    page.getByRole("dialog", { name: "UI framework" }),
  ).toBeAbsent();
  await page.getByRole("option", { name: "SolidJS" }).click();
  await expect(combobox).toBeCollapsed();
  await expect(
    page.getByRole("listbox", { name: "UI framework" }),
  ).toBeAbsent();
  await combobox.click();
  await expect(page.getByRole("option", { name: "SolidJS" })).toBeSelected();
  await expect(page.getByRole("option", { name: "React" })).toBeDeselected();
  await page.getByRole("option", { name: "SolidJS" }).click();

  const region = page.getByRole("combobox", { name: "Deployment region" });
  await region.click();
  await expect(page.getByRole("option", { name: "Hong Kong" })).toBeSelected();
  await page.getByRole("option", { name: "Hong Kong" }).click();
});
