import { renderComponent } from "@wabou/test/component";
import {
  Toolbar,
  ToolbarButton,
  ToolbarGroup,
  ToolbarSeparator,
  ToolbarToggle,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("moves one toolbar tab stop and skips disabled commands", () => {
  const screen = renderComponent(() => (
    <Toolbar aria-label="Editor tools">
      <ToolbarGroup aria-label="History">
        <ToolbarButton aria-label="Undo">Undo</ToolbarButton>
        <ToolbarButton aria-label="Redo" disabled>
          Redo
        </ToolbarButton>
      </ToolbarGroup>
      <ToolbarSeparator />
      <ToolbarButton aria-label="Share">Share</ToolbarButton>
    </Toolbar>
  ));
  const toolbar = screen.getByRole("toolbar", { name: "Editor tools" });
  const undo = toolbar.getByRole("button", { name: "Undo" });
  const share = toolbar.getByRole("button", { name: "Share" });

  expect(toolbar.getAllByRole("button")).toHaveLength(3);
  expect(toolbar.orientation).toBe("horizontal");
  expect(undo.focusOrder).toBe(0);
  expect(share.focusOrder).toBe(-1);
  undo.focus();
  undo.press("ArrowRight");
  expect(share.focused).toBe(true);
  expect(share.focusOrder).toBe(0);
  share.press("Home");
  expect(undo.focused).toBe(true);
});

test("uses vertical arrow keys without consuming cross-axis keys", () => {
  const screen = renderComponent(() => (
    <Toolbar aria-label="Arrange tools" orientation="vertical" loop={false}>
      <ToolbarButton aria-label="Move up" />
      <ToolbarButton aria-label="Move down" />
    </Toolbar>
  ));
  const up = screen.getByRole("button", { name: "Move up" });
  const down = screen.getByRole("button", { name: "Move down" });

  expect(
    screen.getByRole("toolbar", { name: "Arrange tools" }).orientation,
  ).toBe("vertical");
  up.focus();
  up.press("ArrowRight");
  expect(up.focused).toBe(true);
  up.press("ArrowDown");
  expect(down.focused).toBe(true);
  down.press("ArrowDown");
  expect(down.focused).toBe(true);
});

test("supports application-owned toolbar toggles", () => {
  const Controlled = () => {
    const [bold, setBold] = createSignal(false);
    return (
      <Toolbar aria-label="Formatting tools">
        <ToolbarToggle
          aria-label="Bold"
          pressed={bold()}
          onPressedChange={setBold}
        />
      </Toolbar>
    );
  };
  const screen = renderComponent(Controlled);
  const bold = screen.getByRole("button", { name: "Bold" });

  expect(bold.pressed).toBe(false);
  bold.click();
  expect(bold.pressed).toBe(true);
});
