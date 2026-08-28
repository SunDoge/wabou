import { type Host, useHost } from "@wabou/core/renderer";
import { px } from "@wabou/core/style";
import {
  assertFocusOwnerCount,
  assertSingleSurfaceOwner,
  createTestHost,
  renderComponent,
} from "@wabou/test/component";
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

test("reports duplicate surface and unexpected focus ownership", () => {
  const screen = renderComponent(() => (
    <View
      role="group"
      aria-label="Broken compound control"
      data-wabou-owns="surface"
      class="bg-input"
    >
      <Input aria-label="Nested editor" />
    </View>
  ));
  const group = screen.getByRole("group", {
    name: "Broken compound control",
  });

  expect(() => assertSingleSurfaceOwner(group)).toThrow("found 2");
  expect(() => assertFocusOwnerCount(group, 0)).toThrow("found 1");
});

test("rejects visual chrome authored by nested native content", () => {
  const screen = renderComponent(() => (
    <View role="group" aria-label="Compound editor" data-wabou-owns="surface">
      <View
        role="textbox"
        aria-label="Native content"
        data-wabou-owns="native-editor"
        class="rounded-lg bg-input"
      />
    </View>
  ));

  expect(() =>
    assertSingleSurfaceOwner(
      screen.getByRole("group", { name: "Compound editor" }),
    ),
  ).toThrow("must not author visual chrome");
});

test("navigates authored parents and finds a stable semantic ancestor", () => {
  const screen = renderComponent(() => (
    <View role="region" aria-label="Inspector">
      <View role="presentation" class="visual-surface">
        <View role="group" aria-label="Actions">
          <Button aria-label="Save" />
        </View>
      </View>
    </View>
  ));
  const save = screen.getByRole("button", { name: "Save" });

  expect(save.parent?.role).toBe("group");
  expect(save.closestByRole("region", { name: "Inspector" })?.name).toBe(
    "Inspector",
  );
  expect(save.closestByRole("presentation")?.className).toContain(
    "visual-surface",
  );
  expect(save.closestByRole("dialog")).toBeNull();
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
  expect(() => panel.closestByRole("group")).toThrow("detached component");
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

test("observes reactive inline styles emitted through the native protocol", () => {
  let setWidth: ((value: string | undefined) => void) | undefined;
  let useTypedWidth: (() => void) | undefined;
  const StyledPanel = () => {
    const [width, updateWidth] = createSignal<
      string | ReturnType<typeof px> | undefined
    >("40%");
    setWidth = (value) => updateWidth(value);
    useTypedWidth = () => updateWidth(px(96));
    return (
      <View
        role="group"
        aria-label="Styled panel"
        style={{ width: width(), opacity: 0.5 }}
      />
    );
  };
  const screen = renderComponent(StyledPanel);
  const panel = screen.getByRole("group", { name: "Styled panel" });

  expect(panel.style("width")).toBe("40%");
  expect(panel.style("opacity")).toBe("0.5");
  setWidth?.("75%");
  screen.flush();
  expect(panel.style("width")).toBe("75%");
  useTypedWidth?.();
  screen.flush();
  expect(panel.style("width")).toEqual({ kind: 1, value: 96 });
  setWidth?.(undefined);
  screen.flush();
  expect(panel.style("width")).toBeNull();
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

test("distinguishes hover movement from a captured drag", () => {
  const PointerProbe = () => {
    const [buttons, setButtons] = createSignal(-1);
    return (
      <View
        role="group"
        aria-label="Pointer probe"
        onPointerMove={(event: { buttons: number }) =>
          setButtons(event.buttons)
        }
      >
        <Text role="status">{String(buttons())}</Text>
      </View>
    );
  };
  const screen = renderComponent(PointerProbe);
  const probe = screen.getByRole("group", { name: "Pointer probe" });

  probe.movePointer({ offsetX: 12, offsetY: 8 });
  expect(screen.getByRole("status").text).toBe("0");
  probe.pointerMove({ offsetX: 20, offsetY: 8 });
  expect(screen.getByRole("status").text).toBe("1");
});

test("owns delayed component work through an explicit fake clock", async () => {
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
  await screen.advanceTime(49);
  expect(screen.getByRole("status", { name: "Waiting" }).text).toBe("Waiting");
  await screen.advanceTime(1);
  expect(screen.getByRole("status", { name: "Ready" }).text).toBe("Ready");
  await expect(screen.advanceTime(-1)).rejects.toThrow(
    "finite and non-negative",
  );
});

test("waits for Promise-driven component work with bounded diagnostics", async () => {
  const AsyncStatus = () => {
    const [status, setStatus] = createSignal("Idle");
    return (
      <View>
        <Button
          aria-label="Load"
          onClick={async () => {
            await Promise.resolve();
            setStatus("Ready");
          }}
        />
        <Text role="status" aria-label={status()}>
          {status()}
        </Text>
      </View>
    );
  };
  const screen = renderComponent(AsyncStatus, { clock: "fake" });

  screen.getByRole("button", { name: "Load" }).click();
  const ready = await screen.waitFor(() =>
    screen.getByRole("status", { name: "Ready" }),
  );
  expect(ready.text).toBe("Ready");
  await expect(
    screen.waitFor(() => screen.getByRole("status", { name: "Missing" }), {
      timeout: 20,
      interval: 10,
    }),
  ).rejects.toThrow(
    'component wait timed out after 20ms: no component found for role=status name="Missing"',
  );
  await expect(screen.waitFor(() => true, { interval: 0 })).rejects.toThrow(
    "interval must be finite and positive",
  );
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

test("gives injected hosts a deterministic unmeasured layout", () => {
  const fixture = createTestHost();
  const first = { lo: 7, hi: 1 };
  const second = { lo: 8, hi: 1 };

  expect(fixture.host.layout.snapshot([first, { id: second }])).toEqual({
    revision: 0,
    viewport: { x: 0, y: 0, width: 1_024, height: 768 },
    nodes: [
      {
        id: first,
        rect: { x: 0, y: 0, width: 0, height: 0 },
        clip: { x: 0, y: 0, width: 1_024, height: 768 },
        scroll: { offsetX: 0, offsetY: 0, rangeX: 0, rangeY: 0 },
      },
      {
        id: second,
        rect: { x: 0, y: 0, width: 0, height: 0 },
        clip: { x: 0, y: 0, width: 1_024, height: 768 },
        scroll: { offsetX: 0, offsetY: 0, rangeX: 0, rangeY: 0 },
      },
    ],
  });
  expect(fixture.callsTo("layout.snapshot")).toHaveLength(1);
});

test("fails loudly when an unconfigured host side effect is used", () => {
  const fixture = createTestHost();
  expect(() => fixture.host.system.openUrl("https://example.com")).toThrow(
    "test host method system.openUrl",
  );
  expect(fixture.callsTo("system.openUrl")).toHaveLength(1);
});
