import { renderComponent } from "@wabou/test/component";
import { Slider, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";

test("supports keyboard input and publishes range semantics", () => {
  const changes = vi.fn();
  const screen = renderComponent(() => (
    <Slider
      label="Volume"
      defaultValue={25}
      step={5}
      valueText={(value) => `${value} percent`}
      onValueChange={changes}
    />
  ));
  const slider = screen.getByRole("slider", { name: "Volume" });

  expect(slider.numericValue).toBe(25);
  expect(slider.minNumericValue).toBe(0);
  expect(slider.maxNumericValue).toBe(100);
  expect(slider.valueText).toBe("25 percent");

  slider.press("ArrowRight");
  slider.press("PageUp");
  expect(slider.numericValue).toBe(80);
  slider.press("End");
  expect(slider.numericValue).toBe(100);
  slider.press("Home");
  expect(slider.numericValue).toBe(0);
  expect(changes.mock.calls.map(([value]) => value)).toEqual([30, 80, 100, 0]);
});

test("drags against measured width and updates its visual geometry", () => {
  const screen = renderComponent(() => (
    <Slider label="Position" min={10} max={30} step={2} defaultValue={10} />
  ));
  const slider = screen.getByRole("slider", { name: "Position" });
  const [track, thumb] = slider.children;

  slider.resize({ width: 200, height: 28 });
  slider.pointerDown({ offsetX: 51 });
  expect(slider.numericValue).toBe(16);
  expect(track.children[0].style("width")).toBe("30%");
  expect(Number.parseFloat(thumb.style("left") ?? "NaN")).toBeCloseTo(55.2);

  slider.pointerMove({ offsetX: 200 });
  slider.pointerUp({ offsetX: 200 });
  expect(slider.numericValue).toBe(30);
  expect(track.children[0].style("width")).toBe("100%");
  expect(thumb.style("left")).toBe("184px");
});

test("supports controlled values and rejects disabled interaction", () => {
  const Controlled = () => {
    const [value, setValue] = createSignal(2);
    return (
      <View>
        <Slider label="Controlled" value={value()} onValueChange={setValue} />
        <Text role="status" aria-label="Current value">
          {String(value())}
        </Text>
        <Slider label="Unavailable" disabled defaultValue={40} />
      </View>
    );
  };
  const screen = renderComponent(Controlled);
  const controlled = screen.getByRole("slider", { name: "Controlled" });

  controlled.press("ArrowRight");
  expect(controlled.numericValue).toBe(3);
  expect(screen.getByRole("status", { name: "Current value" }).text).toBe("3");

  const disabled = screen.getByRole("slider", { name: "Unavailable" });
  expect(disabled.disabled).toBe(true);
  expect(disabled.focusOrder).toBe(-1);
  expect(() => disabled.press("End")).toThrow(
    'cannot press disabled component slider "Unavailable"',
  );
});
