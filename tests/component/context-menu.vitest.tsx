import { renderComponent } from "@wabou/test/component";
import { Button, ContextMenu } from "@wabou/ui";
import { expect, test } from "vitest";

test("opens from a secondary click and selects an action", () => {
  const actions: string[] = [];
  const screen = renderComponent(() => (
    <ContextMenu
      aria-label="File actions"
      items={[
        { id: "open", label: "Open" },
        { id: "disabled", label: "Disabled", disabled: true },
        { id: "delete", label: "Delete", destructive: true },
      ]}
      onAction={(id) => actions.push(id)}
      trigger={(trigger) => <Button {...trigger}>File</Button>}
    />
  ));

  screen.getByRole("button", { name: "File" }).contextMenu({
    offsetX: 120,
    offsetY: 80,
  });
  const menu = screen.getByRole("menu", { name: "File actions" });
  menu.press("ArrowDown");
  menu.press("Enter");

  expect(actions).toEqual(["delete"]);
  expect(screen.queryByRole("menu")).toBeNull();
});

test("supports keyboard invocation", () => {
  const screen = renderComponent(() => (
    <ContextMenu
      aria-label="File actions"
      items={[{ id: "open", label: "Open" }]}
      trigger={(trigger) => <Button {...trigger}>File</Button>}
    />
  ));

  screen.getByRole("button", { name: "File" }).press("ContextMenu");
  expect(screen.getByRole("menu", { name: "File actions" })).not.toBeNull();
});
