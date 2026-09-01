import { renderComponent } from "@wabou/test/component";
import { Command, CommandList, Text } from "@wabou/ui";
import { expect, test, vi } from "vitest";

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
  const theme = screen.getByRole("option", { name: "Change theme" });
  expect(theme.className).toContain("bg-control-hover");
  expect(theme.className).toContain("rounded-md");
  expect(theme.className).toContain("min-h-8");
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

test("renders shortcut hints without changing the option's accessible name", () => {
  const screen = renderComponent(() => (
    <Command
      aria-label="Commands"
      items={[
        { id: "search", label: "Search conversation", shortcut: "Ctrl F" },
      ]}
    />
  ));

  expect(
    screen.getByRole("option", { name: "Search conversation" }).text,
  ).toContain("Ctrl F");
});

test("reuses command rows for externally controlled completion lists", () => {
  const highlights: string[] = [];
  const actions: string[] = [];
  const screen = renderComponent(() => (
    <CommandList
      aria-label="Completions"
      items={[
        { id: "command:review", label: "/review", description: "Review" },
        { id: "file:README.md", label: "README.md", description: "File" },
      ]}
      highlighted="command:review"
      onHighlightChange={(id) => highlights.push(id)}
      onAction={(id) => actions.push(id)}
      renderLeading={(item) => (
        <Text>{item.id.startsWith("file:") ? "F" : "C"}</Text>
      )}
    />
  ));

  expect(screen.getByRole("listbox", { name: "Completions" })).toBeTruthy();
  expect(screen.getByRole("option", { name: "/review" }).selected).toBe(true);
  const file = screen.getByRole("option", { name: "README.md" });
  file.pointerMove({ clientX: 4, clientY: 4 });
  file.click();
  expect(highlights).toEqual(["file:README.md"]);
  expect(actions).toEqual(["file:README.md"]);
});

test("command list presents exclusive loading and retryable error states", () => {
  const retry = vi.fn();
  const screen = renderComponent(() => (
    <CommandList
      aria-label="Remote commands"
      items={[]}
      error={new Error("offline")}
      errorText="Could not load commands"
      retryLabel="Retry commands"
      onRetry={retry}
    />
  ));

  expect(
    screen.getByRole("alert", { name: "Could not load commands" }).text,
  ).toContain("offline");
  expect(screen.queryByRole("status")).toBeNull();
  screen.getByRole("button", { name: "Retry commands" }).click();
  expect(retry).toHaveBeenCalledOnce();
});
