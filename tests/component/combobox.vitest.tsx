import { renderComponent } from "@wabou/test/component";
import { Combobox, Text } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

const options = [
  { id: "solid", value: "solid", label: "SolidJS", keywords: ["signals"] },
  { id: "disabled", value: "disabled", label: "Disabled", disabled: true },
  { id: "rust", value: "rust", label: "Rust", keywords: ["native"] },
];

test("searches and selects a value", () => {
  const App = () => {
    const [value, setValue] = createSignal("");
    return (
      <>
        <Combobox
          aria-label="Technology"
          options={options}
          onValueChange={setValue}
        />
        <Text role="status">{value()}</Text>
      </>
    );
  };
  const screen = renderComponent(() => <App />);
  const trigger = screen.getByRole("combobox", { name: "Technology" });

  trigger.click();
  expect(
    screen.getByRole("listbox").closestByRole("presentation")?.className,
  ).toContain("rounded-lg");
  screen.getByRole("textbox", { name: "Technology search" }).input("native");
  screen.getByRole("option", { name: "Rust" }).click();

  expect(screen.getByRole("status").text).toBe("rust");
  expect(trigger.text).toContain("Rust");
  expect(trigger.valueText).toBe("Rust");
  expect(screen.queryByRole("listbox")).toBeNull();
});

test("dismisses search with Escape", () => {
  const screen = renderComponent(() => (
    <Combobox aria-label="Technology" options={options} />
  ));
  screen.getByRole("combobox", { name: "Technology" }).click();
  screen.getByRole("textbox", { name: "Technology search" }).press("Escape");
  expect(screen.queryByRole("listbox")).toBeNull();
});

test("visually distinguishes unavailable options", () => {
  const screen = renderComponent(() => (
    <Combobox aria-label="Technology" options={options} />
  ));
  screen.getByRole("combobox", { name: "Technology" }).click();

  const disabled = screen.getByRole("option", { name: "Disabled" });
  expect(disabled.disabled).toBe(true);
  expect(disabled.className).toContain("bg-surface-muted");
  expect(disabled.className).toContain("text-muted");
  expect(disabled.className).toContain("cursor-not-allowed");
  expect(disabled.className).toContain("opacity-60");
});

test("uses neutral disabled chrome instead of an actionable picker surface", () => {
  const screen = renderComponent(() => (
    <Combobox aria-label="Disabled technology" options={options} disabled />
  ));
  const trigger = screen.getByRole("combobox", {
    name: "Disabled technology",
    disabled: true,
  });

  expect(trigger.className).toContain("bg-surface-muted");
  expect(trigger.className).toContain("border-subtle");
  expect(trigger.className).toContain("text-muted");
  expect(trigger.className).toContain("cursor-not-allowed");
  expect(trigger.className).toContain("opacity-60");
});
