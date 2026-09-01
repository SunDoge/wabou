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
