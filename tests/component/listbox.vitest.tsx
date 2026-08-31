import { renderComponent } from "@wabou/test/component";
import { Listbox } from "@wabou/ui";
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
  expect(screen.getByRole("option", { name: "Feature branch" }).selected).toBe(
    true,
  );
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
