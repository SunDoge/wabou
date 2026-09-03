import { renderComponent } from "@wabou/test/component";
import { Listbox, Text } from "@wabou/ui";
import { expect, test, vi } from "vitest";

const options = [
  { value: "main", label: "Main branch" },
  { value: "release", label: "Release branch", disabled: true },
  {
    value: "feature",
    label: "Feature branch",
    description: "Current workspace branch",
  },
] as const;

test("listbox navigates enabled options and activates the highlighted value", () => {
  const action = vi.fn();
  const screen = renderComponent(() => (
    <Listbox aria-label="Branches" options={options} onAction={action} />
  ));
  const listbox = screen.getByRole("listbox", { name: "Branches" });

  expect(listbox.attribute("aria-activedescendant")).toBe("main");
  listbox.press("ArrowDown");
  expect(listbox.attribute("aria-activedescendant")).toBe("feature");
  listbox.press("Enter");

  expect(action).toHaveBeenCalledWith("feature");
  const feature = screen.getByRole("option", { name: "Feature branch" });
  expect(feature.selected).toBe(true);
  expect(feature.className).toContain("rounded-md");
  expect(feature.className).toContain("min-h-8");
});

test("listbox supports pointer selection and explicit dismissal", () => {
  const action = vi.fn();
  const dismiss = vi.fn();
  const screen = renderComponent(() => (
    <Listbox
      aria-label="Branches"
      options={options}
      defaultValue="main"
      onAction={action}
      onDismiss={dismiss}
    />
  ));

  screen.getByRole("option", { name: "Feature branch" }).click();
  expect(action).toHaveBeenCalledWith("feature");
  screen.getByRole("listbox", { name: "Branches" }).press("Escape");
  expect(dismiss).toHaveBeenCalledOnce();
});

test("listbox supports inspector rows with explicit geometry and accessible identities", () => {
  const screen = renderComponent(() => (
    <Listbox
      aria-label="Workspace files"
      viewportHeight={224}
      itemHeight={48}
      options={[
        {
          value: "src/index.ts",
          label: "index.ts",
          accessibilityLabel: "src/index.ts",
          description: "src/index.ts",
        },
      ]}
      renderLeading={() => <Text aria-hidden="true">file</Text>}
    />
  ));

  const option = screen.getByRole("option", { name: "src/index.ts" });
  expect(option.text).toContain("file");
  expect(option.text).toContain("index.ts");
});

test("listbox supports fill layouts and trailing metadata", () => {
  const screen = renderComponent(() => (
    <Listbox
      fill
      aria-label="Skills"
      options={[{ value: "review", label: "Review", description: "Project" }]}
      renderTrailing={() => <Text aria-hidden="true">local</Text>}
    />
  ));

  expect(screen.getByRole("listbox", { name: "Skills" })).toBeDefined();
  expect(screen.getByRole("option", { name: "Review" }).text).toContain(
    "local",
  );
});
