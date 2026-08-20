import { renderComponent } from "@wabou/test/component";
import { Button, createRetainedItems, Text, View } from "@wabou/ui";
import { createSignal, For } from "solid-js";
import { expect, test } from "vitest";

test("retains removed keys until release and keeps active values current", () => {
  let retained: ReturnType<
    typeof createRetainedItems<{ id: string; label: string }, string>
  >;
  const screen = renderComponent(() => {
    const [items, setItems] = createSignal([
      { id: "a", label: "Alpha" },
      { id: "b", label: "Beta" },
    ]);
    retained = createRetainedItems(items, (item) => item.id);
    return (
      <View>
        <Button
          onClick={() =>
            setItems([
              { id: "b", label: "Beta updated" },
              { id: "c", label: "Gamma" },
            ])
          }
        >
          Reconcile
        </Button>
        <For each={retained.entries()}>
          {(entry) => (
            <Text role="status" aria-label={entry.key}>
              {`${entry.present() ? "present" : "exiting"}:${entry.value().label}`}
            </Text>
          )}
        </For>
      </View>
    );
  });

  screen.getByRole("button", { name: "Reconcile" }).click();
  expect(screen.getByRole("status", { name: "a" }).text).toBe("exiting:Alpha");
  expect(screen.getByRole("status", { name: "b" }).text).toBe(
    "present:Beta updated",
  );
  expect(screen.getByRole("status", { name: "c" }).text).toBe("present:Gamma");

  expect(retained.release("b")).toBe(false);
  expect(retained.release("a")).toBe(true);
  screen.flush();
  expect(screen.queryByRole("status", { name: "a" })).toBeNull();
});

test("rejects duplicate source keys", () => {
  expect(() =>
    renderComponent(() => {
      const [items] = createSignal([
        { id: "same", label: "First" },
        { id: "same", label: "Second" },
      ]);
      createRetainedItems(items, (item) => item.id);
      return null;
    }),
  ).toThrow("duplicate retained item key: same");
});

test("reuses an exiting entry when its key returns", () => {
  let retained!: ReturnType<
    typeof createRetainedItems<{ id: string; label: string }, string>
  >;
  const screen = renderComponent(() => {
    const [items, setItems] = createSignal([{ id: "a", label: "Alpha" }]);
    retained = createRetainedItems(items, (item) => item.id);
    return (
      <View>
        <Button onClick={() => setItems([])}>Remove</Button>
        <Button
          onClick={() => setItems([{ id: "a", label: "Alpha restored" }])}
        >
          Restore
        </Button>
      </View>
    );
  });

  const original = retained.entries()[0];
  screen.getByRole("button", { name: "Remove" }).click();
  expect(original.present()).toBe(false);
  screen.getByRole("button", { name: "Restore" }).click();
  expect(retained.entries()[0]).toBe(original);
  expect(original.present()).toBe(true);
  expect(original.value().label).toBe("Alpha restored");
});
