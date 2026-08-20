import { type Host, useHost } from "@wabou/core/renderer";
import { createTestHost, renderComponent } from "@wabou/test/component";
import {
  Button,
  createMeasuredSize,
  Input,
  Slider,
  Text,
  View,
} from "@wabou/ui";
import { createSignal, Show } from "solid-js";
import { expect, test } from "vitest";

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

test("scopes role queries to a component subtree", () => {
  const screen = renderComponent(() => (
    <View>
      <View role="group" aria-label="Profile form">
        <Button aria-label="Save" />
        <Button aria-label="Cancel" />
      </View>
      <View role="group" aria-label="Security form">
        <Button aria-label="Save" />
      </View>
    </View>
  ));
  const forms = screen.getAllByRole("group");
  const profile = screen.getByRole("group", { name: "Profile form" });
  const security = screen.getByRole("group", { name: "Security form" });

  expect(forms).toHaveLength(2);
  expect(screen.queryAllByRole("button")).toHaveLength(3);
  expect(profile.getAllByRole("button")).toHaveLength(2);
  expect(profile.getByRole("button", { name: "Save" }).name).toBe("Save");
  expect(security.queryByRole("button", { name: "Cancel" })).toBeNull();
  expect(() => screen.getByRole("button", { name: "Save" })).toThrow(
    "use getAllByRole",
  );
});

test("scoped queries follow dynamic children and reject detached roots", () => {
  const Dynamic = () => {
    const [child, setChild] = createSignal(true);
    const [panel, setPanel] = createSignal(true);
    return (
      <View>
        <Button aria-label="Remove child" onClick={() => setChild(false)} />
        <Button aria-label="Remove panel" onClick={() => setPanel(false)} />
        <Show when={panel()}>
          <View role="group" aria-label="Dynamic panel">
            <Show when={child()}>
              <Button aria-label="Dynamic action" />
            </Show>
          </View>
        </Show>
      </View>
    );
  };
  const screen = renderComponent(Dynamic);
  const panel = screen.getByRole("group", { name: "Dynamic panel" });

  expect(panel.getAllByRole("button")).toHaveLength(1);
  screen.getByRole("button", { name: "Remove child" }).click();
  expect(panel.queryAllByRole("button")).toHaveLength(0);
  screen.getByRole("button", { name: "Remove panel" }).click();
  expect(() => panel.queryAllByRole("button")).toThrow("detached component");
  expect(() => panel.resize({ width: 100, height: 40 })).toThrow(
    "resize detached component",
  );
});

test("drives a real component through keyboard events", () => {
  const screen = renderComponent(() => (
    <Slider label="Volume" defaultValue={40} step={5} />
  ));
  const slider = screen.getByRole("slider", { name: "Volume" });

  slider.press("ArrowRight");

  expect(slider.numericValue).toBe(45);
});

test("observes native interaction policy instead of treating it as attributes", () => {
  const screen = renderComponent(() => (
    <View
      role="group"
      aria-label="Modal region"
      focusOrder={3}
      interactionBlocked
      focusContained
    />
  ));
  const region = screen.getByRole("group", { name: "Modal region" });

  expect(region.focusOrder).toBe(3);
  expect(region.interactionBlocked).toBe(true);
  expect(region.focusContained).toBe(true);
  expect(region.attribute("focusOrder")).toBeNull();
});

test("reads reactive semantic states without asserting raw protocol attributes", () => {
  const SemanticStates = () => {
    const [checked, setChecked] = createSignal(false);
    return (
      <View>
        <Button
          role="checkbox"
          aria-label="Updates"
          aria-checked={checked()}
          onClick={() => setChecked((value) => !value)}
        />
        <Button aria-label="Unavailable" disabled />
        <View role="tab" aria-label="General" aria-selected="true" />
        <View role="link" aria-label="Overview" aria-current="page" />
        <View
          role="toolbar"
          aria-label="Editing tools"
          aria-orientation="vertical"
        />
        <View role="group" aria-label="Details" aria-expanded="false" />
        <View role="button" aria-label="Bold" aria-pressed="mixed" />
        <View
          role="slider"
          aria-label="Volume"
          aria-valuemin="0"
          aria-valuemax="100"
          aria-valuenow="42.5"
          aria-valuetext="42.5 percent"
        />
      </View>
    );
  };
  const screen = renderComponent(SemanticStates);
  const updates = screen.getByRole("checkbox", {
    name: "Updates",
    checked: false,
  });

  expect(updates.checked).toBe(false);
  updates.click();
  expect(updates.checked).toBe(true);
  expect(screen.getByRole("checkbox", { checked: true }).name).toBe("Updates");
  expect(screen.getByRole("button", { disabled: true }).name).toBe(
    "Unavailable",
  );
  expect(
    screen.getAllByRole("button", { disabled: false }).map((item) => item.name),
  ).toEqual(["Bold"]);
  expect(screen.getByRole("tab", { selected: true }).name).toBe("General");
  expect(screen.getByRole("link", { current: "page" }).name).toBe("Overview");
  expect(screen.getByRole("toolbar", { orientation: "vertical" }).name).toBe(
    "Editing tools",
  );
  expect(screen.getByRole("group", { name: "Details" }).expanded).toBe(false);
  expect(screen.getByRole("button", { name: "Bold" }).pressed).toBe("mixed");
  const volume = screen.getByRole("slider", { name: "Volume" });
  expect(volume.numericValue).toBe(42.5);
  expect(volume.minNumericValue).toBe(0);
  expect(volume.maxNumericValue).toBe(100);
  expect(volume.valueText).toBe("42.5 percent");
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

test("observes runtime transforms without decoding protocol bytes", () => {
  const screen = renderComponent(() => (
    <View
      role="group"
      aria-label="Animated panel"
      transform={[1, 0, 0, 1, 24, -8]}
    />
  ));

  expect(
    screen.getByRole("group", { name: "Animated panel" }).transform,
  ).toEqual([1, 0, 0, 1, 24, -8]);
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

test("owns delayed component work through an explicit fake clock", () => {
  const DelayedStatus = () => {
    const [status, setStatus] = createSignal("Idle");
    return (
      <View>
        <Button
          aria-label="Start"
          onClick={() => {
            setStatus("Waiting");
            setTimeout(() => setStatus("Ready"), 50);
          }}
        />
        <Text role="status" aria-label={status()}>
          {status()}
        </Text>
      </View>
    );
  };
  const screen = renderComponent(DelayedStatus, { clock: "fake" });

  screen.getByRole("button", { name: "Start" }).click();
  expect(screen.getByRole("status", { name: "Waiting" }).text).toBe("Waiting");
  screen.advanceTime(49);
  expect(screen.getByRole("status", { name: "Waiting" }).text).toBe("Waiting");
  screen.advanceTime(1);
  expect(screen.getByRole("status", { name: "Ready" }).text).toBe("Ready");
  expect(() => screen.advanceTime(-1)).toThrow("finite and non-negative");
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
