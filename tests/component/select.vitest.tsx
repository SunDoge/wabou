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
  expect(
    screen.getByRole("listbox", { name: "Technology" }).className,
  ).toContain("select-none");
  rust.movePointer();
  expect(rust.className).toContain("bg-control-hover");
  expect(() =>
    screen.getByRole("option", { name: "Unavailable" }).click(),
  ).toThrow("cannot click disabled component option");
  const unavailable = screen.getByRole("option", { name: "Unavailable" });
  expect(unavailable.className).toContain("bg-surface-muted");
  expect(unavailable.className).toContain("text-muted");
  expect(unavailable.className).toContain("cursor-not-allowed");
  expect(unavailable.className).toContain("opacity-60");

  rust.click();
  expect(screen.getByRole("status").text).toBe("rust");
  expect(trigger.text).toContain("Rust");
  expect(trigger.valueText).toBe("Rust");
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

test("uses neutral disabled chrome instead of an actionable picker surface", () => {
  const screen = renderComponent(() => (
    <Select aria-label="Disabled technology" options={options} disabled />
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
  expect(() => trigger.click()).toThrow("cannot click disabled component");
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

test("does not move focus when a controlled owner rejects an open request", async () => {
  const requests: boolean[] = [];
  const screen = renderComponent(
    () => (
      <Select
        aria-label="Controlled closed technology"
        options={options}
        open={false}
        onOpenChange={(open) => requests.push(open)}
      />
    ),
    { clock: "fake" },
  );
  const trigger = screen.getByRole("combobox", {
    name: "Controlled closed technology",
  });
  trigger.focus();
  trigger.press("ArrowDown");
  await screen.advanceTime(16);

  expect(requests).toEqual([true]);
  expect(trigger.expanded).toBe(false);
  expect(trigger.focused).toBe(true);
  expect(screen.queryByRole("listbox")).toBeNull();
});

test("keeps popup focus when a controlled owner rejects dismissal", async () => {
  const requests: boolean[] = [];
  const screen = renderComponent(
    () => (
      <Select
        aria-label="Controlled open technology"
        options={options}
        open
        onOpenChange={(open) => requests.push(open)}
      />
    ),
    { clock: "fake" },
  );
  await screen.advanceTime(16);
  const trigger = screen.getByRole("combobox", {
    name: "Controlled open technology",
  });
  const listbox = screen.getByRole("listbox", {
    name: "Controlled open technology",
  });
  expect(listbox.focused).toBe(true);

  listbox.press("Escape");
  await screen.advanceTime(16);

  expect(requests).toEqual([false]);
  expect(trigger.expanded).toBe(true);
  expect(listbox.focused).toBe(true);
  expect(trigger.focused).toBe(false);
});

test("opens at the first enabled option when the controlled value is stale", async () => {
  const screen = renderComponent(
    () => (
      <Select aria-label="Stale technology" options={options} value="removed" />
    ),
    { clock: "fake" },
  );
  screen.getByRole("combobox", { name: "Stale technology" }).press("ArrowDown");
  await screen.advanceTime(16);

  expect(screen.getByRole("option", { name: "SolidJS" }).className).toContain(
    "bg-control-hover",
  );
});

test("opens on pointer down without waiting for the pressed state to release", () => {
  const screen = renderComponent(() => (
    <Select aria-label="Technology" options={options} defaultValue="solid" />
  ));
  const trigger = screen.getByRole("combobox", { name: "Technology" });

  trigger.pointerDown();
  expect(trigger.expanded).toBe(true);
  expect(screen.getByRole("listbox", { name: "Technology" })).not.toBeNull();

  trigger.pointerUp();
  // The synthetic click following the pointer gesture must not immediately
  // close the panel opened by pointer-down.
  trigger.click();
  expect(trigger.expanded).toBe(true);
});

test("opens without implicit motion by default", () => {
  const screen = renderComponent(() => (
    <Select aria-label="Technology" options={options} defaultValue="solid" />
  ));

  screen.getByRole("combobox", { name: "Technology" }).click();
  const panel = screen.getByRole("listbox").closestByRole("presentation");
  expect(panel?.transform).toEqual([1, 0, 0, 1, 0, 0]);
  screen.getByRole("option", { name: "Rust" }).click();
  expect(screen.getByRole("combobox").text).toContain("Rust");
});

test("supports explicit popup motion", () => {
  const screen = renderComponent(() => (
    <Select
      aria-label="Technology"
      options={options}
      defaultValue="solid"
      motion={{ duration: 0.1, fromScale: 0.98 }}
    />
  ));

  screen.getByRole("combobox").click();
  const panel = screen.getByRole("listbox").closestByRole("presentation");
  expect(panel?.transform).toEqual([1, 0, 0, 1, 0, 0]);
  expect(panel?.attribute("__wabou_native_transition")).toBeNull();
});
