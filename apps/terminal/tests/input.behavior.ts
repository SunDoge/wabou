import { expect, test } from "@wabou/test";

test("terminal receives native focus and committed keyboard text", async ({
  page,
}) => {
  const terminal = page.getByRole("textbox", { name: "Shell 1 terminal" });
  await terminal.click();
  await expect(terminal).toBeFocused();
  await terminal.type("printf wabou-terminal-input");
  await terminal.press("Enter");
  await expect(terminal).toBeFocused();
});
