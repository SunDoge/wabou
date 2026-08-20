import { expect, test } from "vitest";
import { createSignal } from "solid-js";
import { Button, createMeasuredSize, Slider, Text, View } from "@wabou/ui";
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

test("drives a real component through keyboard events", () => {
  const screen = renderComponent(() => (
    <Slider label="Volume" defaultValue={40} step={5} />
  ));
  const slider = screen.getByRole("slider", { name: "Volume" });

  slider.press("ArrowRight");

  expect(slider.attribute("aria-valuenow")).toBe("45");
});

test("publishes deterministic native measurements to a component", () => {
  const MeasuredPanel = () => {
    const size = createMeasuredSize();
    return (
      <View ref={size.ref} role="group" aria-label="Panel">
        <Text role="status" aria-label={`Width ${size.width()}`}>
          {String(size.width())}
        </Text>
      </View>
    );
  };
  const screen = renderComponent(() => <MeasuredPanel />);
  const panel = screen.getByRole("group", { name: "Panel" });

  panel.resize({ width: 240, height: 28 });

  expect(screen.getByRole("status", { name: "Width 240" }).text).toBe("240");
});
