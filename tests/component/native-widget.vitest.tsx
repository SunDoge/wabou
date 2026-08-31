import { renderComponent } from "@wabou/test/component";
import { NativeWidget, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("closes the controlled native widget config and event loop", () => {
  const Meter = () => {
    const [value, setValue] = createSignal(2);
    return (
      <View>
        <NativeWidget
          tag="meter"
          role="slider"
          aria-label="Intensity"
          aria-valuemin={0}
          aria-valuemax={10}
          aria-valuenow={value()}
          config={{ value: value(), accent: "violet" }}
          onChange={(event) => setValue(event.value)}
        />
        <Text role="status" aria-label="Current intensity">
          {String(value())}
        </Text>
      </View>
    );
  };
  const screen = renderComponent(Meter);
  const meter = screen.getByRole("slider", { name: "Intensity" });
  const identity = meter.identity;

  expect(meter.widgetConfig).toEqual({ value: 2, accent: "violet" });
  meter.emit("change", { value: 8 });

  const updated = screen.getByRole("slider", { name: "Intensity" });
  expect(updated.identity).toEqual(identity);
  expect(updated.numericValue).toBe(8);
  expect(updated.widgetConfig).toEqual({ value: 8, accent: "violet" });
  expect(screen.getByRole("status", { name: "Current intensity" }).text).toBe(
    "8",
  );
});

test("uses the ordinary controlled input contract for a custom native editor", () => {
  const NativeEditor = () => {
    const [value, setValue] = createSignal("initial");
    return (
      <NativeWidget
        tag="native-editor"
        role="textbox"
        aria-label="Native source"
        config={{ value: value() }}
        onInput={(event) => setValue(event.currentTarget.value)}
      />
    );
  };
  const screen = renderComponent(NativeEditor);
  const editor = screen.getByRole("textbox", { name: "Native source" });
  const identity = editor.identity;

  editor.input("updated by GPUI");

  const updated = screen.getByRole("textbox", { name: "Native source" });
  expect(updated.identity).toEqual(identity);
  expect(updated.widgetConfig).toEqual({ value: "updated by GPUI" });
});
