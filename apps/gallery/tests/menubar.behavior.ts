import { expect, test } from "@wabou/test";

test("menubar switches sibling menus and dispatches an action", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Menubar" }).click();
  await page.getByRole("menubar", { name: "Editor menu" }).waitFor();

  await page.getByRole("menuitem", { name: "File" }).click();
  const fileMenu = page.getByRole("menu", { name: "File menu" });
  await fileMenu.waitFor();
  await page.getByRole("menuitem", { name: "Edit" }).click();
  await expect(fileMenu).toBeAbsent();

  const editMenu = page.getByRole("menu", { name: "Edit menu" });
  await editMenu.waitFor();
  await editMenu.press("ArrowLeft");
  await expect(editMenu).toBeAbsent();
  await page.getByRole("menu", { name: "File menu" }).press("ArrowRight");
  await editMenu.waitFor();
  await editMenu.press("End");
  await editMenu.press("Enter");
  await expect(
    page.getByRole("status", { name: "Last menu command" }),
  ).toHaveText("paste");
});
