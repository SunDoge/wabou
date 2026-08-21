import { renderComponent } from "@wabou/test/component";
import { SearchField, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("updates an uncontrolled query and clears it with Escape", () => {
  let changed = "";
  const screen = renderComponent(() => (
    <SearchField
      aria-label="Search files"
      defaultValue="src"
      onValueChange={(value) => (changed = value)}
    />
  ));
  const input = screen.getByRole("textbox", { name: "Search files" });

  input.input("packages");
  expect(changed).toBe("packages");
  input.press("Escape");
  expect(changed).toBe("");
  expect(input.focused).toBe(true);
  expect(screen.queryByRole("button", { name: "Clear search" })).toBeNull();
});

test("clears a controlled query and restores input focus", () => {
  const Controlled = () => {
    const [query, setQuery] = createSignal("wabou");
    return (
      <View>
        <SearchField
          aria-label="Search projects"
          value={query()}
          onValueChange={setQuery}
          clearLabel="Reset project search"
        />
        <Text role="status">{query()}</Text>
      </View>
    );
  };
  const screen = renderComponent(Controlled);
  screen.getByRole("button", { name: "Reset project search" }).click();

  expect(screen.getByRole("status").text).toBe("");
  expect(screen.getByRole("textbox", { name: "Search projects" }).focused).toBe(
    true,
  );
});

test("submits the current query with Enter", () => {
  let submitted = "";
  const screen = renderComponent(() => (
    <SearchField
      aria-label="Search commands"
      defaultValue="deploy"
      onSearch={(value) => (submitted = value)}
    />
  ));

  screen.getByRole("textbox", { name: "Search commands" }).press("Enter");
  expect(submitted).toBe("deploy");
});

test("uses one configurable surface for the compound input", () => {
  const screen = renderComponent(() => (
    <SearchField
      aria-label="Search files"
      surfaceClass="bg-surface-raised"
    />
  ));
  const input = screen.getByRole("textbox", { name: "Search files" });
  const group = input.parent;

  expect(group?.className).toContain("bg-surface-raised");
  expect(group?.className).not.toContain("bg-input");
  expect(input.className).not.toContain("bg-input");
  expect(input.className).not.toContain("bg-transparent");
  expect(input.className).not.toContain("border-subtle");
});
