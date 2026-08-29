import { expect, test } from "@wabou/test";

test("opens and closes the application command palette from the primary shortcut", async ({
  page,
}) => {
  const workspace = page.getByRole("textbox", { name: "Workspace" });
  await workspace.press("k", { control: true });

  const palette = page.getByRole("dialog", { name: "Command palette" });
  await expect(palette).toBeInViewport();
  await expect(
    palette.getByRole("option", { name: "New session" }),
  ).toBeInViewport();

  await palette
    .getByRole("textbox", { name: "Command palette" })
    .press("Escape");
  await expect(palette).toBeAbsent();
});
