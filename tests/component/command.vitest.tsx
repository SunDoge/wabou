import { renderComponent } from "@wabou/test/component";
import { Command } from "@wabou/ui";
import { expect, test } from "vitest";

const items = [
  { id: "open", label: "Open project", keywords: ["folder"] },
  { id: "locked", label: "Locked action", disabled: true },
  { id: "theme", label: "Change theme", keywords: ["dark appearance"] },
];

test("filters commands and selects the highlighted result", () => {
  const actions: string[] = [];
  const screen = renderComponent(() => (
    <Command
      aria-label="Commands"
      items={items}
      onAction={(id) => actions.push(id)}
    />
  ));
  const input = screen.getByRole("textbox", { name: "Commands" });

  input.input("dark");
  expect(screen.queryByRole("option", { name: "Open project" })).toBeNull();
  expect(
    screen.getByRole("option", { name: "Change theme" }).className,
  ).toContain("bg-control-hover");
  input.press("Enter");
  expect(actions).toEqual(["theme"]);
});

test("keyboard navigation skips disabled commands", () => {
  const screen = renderComponent(() => (
    <Command aria-label="Commands" items={items} />
  ));
  const input = screen.getByRole("textbox", { name: "Commands" });

  input.press("ArrowDown");
  expect(
    screen.getByRole("option", { name: "Change theme" }).className,
  ).toContain("bg-control-hover");
});

test("renders a semantic empty state", () => {
  const screen = renderComponent(() => (
    <Command aria-label="Commands" items={items} />
  ));
  screen.getByRole("textbox", { name: "Commands" }).input("missing");
  expect(screen.getByRole("status").text).toBe("No results found.");
});
