import { expect, test } from "@wabou/test";

test("semantic locator drives keyboard, text, paste, IME, drag, and wheel input", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Input" }).click();
  const input = page.getByRole("textbox", { name: "Workspace name" });
  await input.dragBy(24, 0);
  await input.wheel(44);
  await input.press("ArrowLeft", { shift: true });
  await input.type("abc");
  await input.paste("中");
  await input.ime("你");
  await expect(input).toBeFocused();
  await page.getByRole("label", { name: "Value: abc中你" }).waitFor();
  const editor = page.getByRole("textbox", { name: "JSON configuration" });
  await editor.press("End");
  await editor.type(" ");
  await editor.paste("// edited");
  await expect(editor).toBeFocused();
  await expect(input).toBeBlurred();
  await expect(editor).toHaveValue(
    '{ // edited\n  "enabled": true\n}',
  );
  await page.getByRole("label", { name: "Config edited" }).waitFor();
});
