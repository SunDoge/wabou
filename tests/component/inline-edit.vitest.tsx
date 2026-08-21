import { renderComponent } from "@wabou/test/component";
import { InlineEdit } from "@wabou/ui";
import { expect, test, vi } from "vitest";

test("commits and cancels desktop inline edits explicitly", () => {
  const onValueChange = vi.fn();
  const onCommit = vi.fn();
  const onCancel = vi.fn();
  const screen = renderComponent(() => (
    <InlineEdit
      aria-label="File name"
      defaultValue="notes.md"
      onValueChange={onValueChange}
      onCommit={onCommit}
      onCancel={onCancel}
    />
  ));

  screen.getByRole("button", { name: "Edit File name" }).click();
  const firstInput = screen.getByRole("textbox", { name: "File name" });
  expect(firstInput.focused).toBe(true);
  firstInput.input("design.md");
  firstInput.press("Enter");
  expect(onValueChange).toHaveBeenCalledWith("design.md");
  expect(onCommit).toHaveBeenCalledWith("design.md");
  expect(screen.getByRole("button", { name: "Edit File name" }).text).toContain(
    "design.md",
  );

  screen.getByRole("button", { name: "Edit File name" }).click();
  expect(screen.queryByRole("button", { name: "Edit File name" })).toBeNull();
  const secondInput = screen.getByRole("textbox", { name: "File name" });
  secondInput.input("discarded.md");
  secondInput.press("Escape");
  expect(onCancel).toHaveBeenCalledOnce();
  expect(onValueChange).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("button", { name: "Edit File name" }).text).toContain(
    "design.md",
  );
});
