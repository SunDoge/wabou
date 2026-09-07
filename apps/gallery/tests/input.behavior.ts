import { expect, test } from "@wabou/test";

const ARABIC_SHAPING_SAMPLE =
  "Shaping Test · اللغة العربية الفصحى تحتاج إلى تشكيل معقد";

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
  await expect(editor).toHaveValue('{ // edited\n  "enabled": true\n}');
  await page.getByRole("label", { name: "Config edited" }).waitFor();
});

test("mixed complex scripts retain text and replace a bidi selection", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Input" }).click();
  await page
    .getByRole("status", { name: "Tibetan complex shaping sample" })
    .waitFor();

  const input = page.getByRole("textbox", {
    name: "Arabic shaping and selection",
  });
  await expect(input).toHaveValue(ARABIC_SHAPING_SAMPLE);
  // Exercise both platform primary-modifier conventions. The non-primary
  // chord is ignored, while Linux/Windows select on Control and macOS selects
  // on Meta.
  await input.press("a", { control: true });
  await input.press("a", { meta: true });
  await input.type(ARABIC_SHAPING_SAMPLE);
  await expect(input).toHaveValue(ARABIC_SHAPING_SAMPLE);
  await expect(input).toBeFocused();
});
