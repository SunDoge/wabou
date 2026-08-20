import { renderComponent } from "@wabou/test/component";
import { Select, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

const options = [
  { value: "solid", label: "SolidJS" },
  { value: "disabled", label: "Unavailable", disabled: true },
  { value: "rust", label: "Rust" },
] as const;

test("opens, highlights, selects, and restores trigger focus", async () => {
  const App = () => {
    const [value, setValue] = createSignal("solid");
    return (
      <View>
        <Select
          aria-label="Technology"
          options={options}
          value={value()}
          onValueChange={setValue}
        />
        <Text role="status">{value()}</Text>
      </View>
    );
  };
  const screen = renderComponent(App, { clock: "fake" });
  const trigger = screen.getByRole("combobox", { name: "Technology" });

  expect(trigger.expanded).toBe(false);
  trigger.click();
  expect(trigger.expanded).toBe(true);
  expect(screen.getByRole("option", { selected: true }).name).toBe("SolidJS");
  await screen.advanceTime(16);
  expect(screen.getByRole("listbox", { name: "Technology" }).focused).toBe(
    true,
  );

  const rust = screen.getByRole("option", { name: "Rust" });
  rust.movePointer();
  expect(rust.className).toContain("bg-control-hover");
  expect(() =>
    screen.getByRole("option", { name: "Unavailable" }).click(),
  ).toThrow("cannot click disabled component option");

  rust.click();
  expect(screen.getByRole("status").text).toBe("rust");
  expect(trigger.text).toContain("Rust");
  expect(trigger.expanded).toBe(false);
  expect(screen.queryByRole("listbox")).toBeNull();
  await screen.advanceTime(16);
  expect(trigger.focused).toBe(true);
});

test("navigates with the keyboard while skipping disabled options", async () => {
  const screen = renderComponent(
    () => (
      <Select aria-label="Technology" options={options} defaultValue="solid" />
    ),
    { clock: "fake" },
  );
  const trigger = screen.getByRole("combobox", { name: "Technology" });

  trigger.press("ArrowDown");
  await screen.advanceTime(16);
  const listbox = screen.getByRole("listbox", { name: "Technology" });
  expect(listbox.focused).toBe(true);
  listbox.press("ArrowDown");
  expect(screen.getByRole("option", { name: "Rust" }).className).toContain(
    "bg-control-hover",
  );
  listbox.press("Enter");
  expect(trigger.text).toContain("Rust");
  expect(screen.queryByRole("listbox")).toBeNull();
});

test("keeps controlled open state owned by the application", () => {
  const App = () => {
    const [open, setOpen] = createSignal(false);
    return (
      <View>
        <Select
          aria-label="Technology"
          options={options}
          open={open()}
          onOpenChange={setOpen}
        />
        <Text role="status">{open() ? "open" : "closed"}</Text>
      </View>
    );
  };
  const screen = renderComponent(App);
  const trigger = screen.getByRole("combobox", { name: "Technology" });

  trigger.click();
  expect(screen.getByRole("status").text).toBe("open");
  expect(trigger.expanded).toBe(true);
  screen.getByRole("listbox", { name: "Technology" }).press("Escape");
  expect(screen.getByRole("status").text).toBe("closed");
  expect(trigger.expanded).toBe(false);
});
