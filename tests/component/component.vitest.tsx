import { expect, test } from "vitest";
import { createSignal } from "solid-js";
import {
  Button,
  createMeasuredSize,
  Input,
  Slider,
  Text,
  View,
} from "@wabou/ui";
import { createTestHost, renderComponent } from "@wabou/test/component";
import { useHost, type Host } from "@wabou/core/renderer";

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

test("enters text through the real controlled-input event path", () => {
  const Form = () => {
    const [name, setName] = createSignal("");
    return (
      <View>
        <Input
          aria-label="Name"
          value={name()}
          onInput={(event) => setName(event.currentTarget.value)}
        />
        <Text role="status" aria-label={`Name ${name()}`}>
          {name()}
        </Text>
      </View>
    );
  };
  const screen = renderComponent(() => <Form />);

  screen.getByRole("textbox", { name: "Name" }).input("Ada");

  expect(screen.getByRole("status", { name: "Name Ada" }).text).toBe("Ada");
});

test("observes hover styling without intercepting the protocol writer", () => {
  const screen = renderComponent(() => (
    <Button variant="ghost" aria-label="Menu">
      Menu
    </Button>
  ));
  const button = screen.getByRole("button", { name: "Menu" });
  expect(button.className).toContain("bg-transparent");

  button.hover();

  expect(button.className).toContain("bg-control-hover");
});

test("uses the native pointer sequence and exposes transient press state", () => {
  let activations = 0;
  const screen = renderComponent(() => (
    <Button variant="secondary" aria-label="Run" onClick={() => activations++}>
      Run
    </Button>
  ));
  const button = screen.getByRole("button", { name: "Run" });

  button.pointerDown({ offsetX: 8, offsetY: 4 });
  expect(button.className).toContain("bg-control-pressed");
  button.pointerUp({ offsetX: 8, offsetY: 4 });
  expect(activations).toBe(0);

  button.click();
  expect(activations).toBe(1);
  expect(button.className).not.toContain("bg-control-pressed");
});

test("injects typed host capabilities and records their calls", () => {
  interface DemoCapability {
    demo: {
      format(request: { value: number }): { label: string };
    };
  }
  const fixture = createTestHost<DemoCapability>({
    demo: {
      format: ({ value }) => ({ label: `Value ${value}` }),
    },
  });
  const HostConsumer = () => {
    const host = useHost<Host & DemoCapability>();
    return <Text role="status">{host.demo.format({ value: 42 }).label}</Text>;
  };

  const screen = renderComponent(() => <HostConsumer />, {
    host: fixture.host,
  });

  expect(screen.getByRole("status", { name: "Value 42" }).text).toBe(
    "Value 42",
  );
  expect(fixture.callsTo("demo.format")).toEqual([
    { path: "demo.format", args: [{ value: 42 }] },
  ]);
});

test("fails loudly when an unconfigured host side effect is used", () => {
  const fixture = createTestHost();
  expect(() => fixture.host.system.openUrl("https://example.com")).toThrow(
    "test host method system.openUrl",
  );
  expect(fixture.callsTo("system.openUrl")).toHaveLength(1);
});
