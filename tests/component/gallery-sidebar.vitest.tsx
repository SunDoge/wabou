import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { GallerySidebar } from "../../apps/gallery/ui/sidebar";

const groups = [
  {
    label: "Actions",
    items: [
      { id: "button", name: "Button" },
      { id: "dialog", name: "Dialog" },
    ],
  },
  {
    label: "Forms",
    items: [{ id: "input", name: "Input" }],
  },
] as const;

const descriptions = {
  button: "Runs an action",
  dialog: "Shows modal content",
  input: "Edits text values",
};

test("filters navigation by name, group, and description", () => {
  const screen = renderComponent(() => (
    <GallerySidebar
      groups={groups}
      descriptions={descriptions}
      selected={null}
      onSelect={() => {}}
    />
  ));
  const search = screen.getByRole("textbox", { name: "Search components" });

  search.input("modal");
  expect(screen.getByRole("button", { name: "Dialog" })).not.toBeNull();
  expect(screen.queryByRole("button", { name: "Button" })).toBeNull();

  search.input("forms");
  expect(screen.getByRole("button", { name: "Input" })).not.toBeNull();
  expect(screen.queryByRole("button", { name: "Dialog" })).toBeNull();
});

test("shows an empty state and clears with Escape or the clear button", () => {
  const screen = renderComponent(() => (
    <GallerySidebar
      groups={groups}
      descriptions={descriptions}
      selected={null}
      onSelect={() => {}}
    />
  ));
  const search = screen.getByRole("textbox", { name: "Search components" });

  search.input("missing");
  expect(
    screen.getByRole("status", { name: "No components found" }),
  ).not.toBeNull();
  search.press("Escape");
  expect(screen.getByRole("button", { name: "Overview" })).not.toBeNull();

  search.input("button");
  screen.getByRole("button", { name: "Clear component search" }).click();
  expect(screen.getByRole("button", { name: "Input" })).not.toBeNull();
});

test("leaves routing ownership with the gallery", () => {
  let selected: string | null = null;
  const screen = renderComponent(() => (
    <GallerySidebar
      groups={groups}
      descriptions={descriptions}
      selected={selected as "button" | "dialog" | "input" | null}
      onSelect={(id) => (selected = id)}
    />
  ));

  screen.getByRole("button", { name: "Button" }).click();
  expect(selected).toBe("button");
});
