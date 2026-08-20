import { expect, test } from "vitest";
import { createSignal } from "solid-js";
import { Button, Text, View } from "@wabou/ui";
import { renderComponent } from "@wabou/test/component";

test("tests a reactive component through its authored role and name", () => {
  const Counter = () => {
    const [count, setCount] = createSignal(0);
    return (
      <View>
        <Button
          aria-label="Increment"
          onClick={() => setCount((value) => value + 1)}
        >
          Increment
        </Button>
        <Text role="status" aria-label={`Count ${count()}`}>
          {String(count())}
        </Text>
      </View>
    );
  };
  const screen = renderComponent(Counter);
  screen.getByRole("button", { name: "Increment" }).click();
  expect(screen.getByRole("status", { name: "Count 1" }).text).toBe("1");
});

test("strict queries reject ambiguous components", () => {
  const screen = renderComponent(() => (
    <View>
      <Button aria-label="Save" />
      <Button aria-label="Save" />
    </View>
  ));
  expect(() => screen.getByRole("button", { name: "Save" })).toThrow(
    "found 2 matches",
  );
  expect(screen.getByRole("button", { name: "Save", index: 1 }).tag).toBe(
    "button",
  );
});
