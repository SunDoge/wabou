import { renderComponent } from "@wabou/test/component";
import { MotionConfigProvider, Switch } from "@wabou/ui";
import { expect, test } from "vitest";
import { switchControlClass } from "../../packages/ui/src/components/switch";

const thumbX = (screen: ReturnType<typeof renderComponent>): number => {
  const control = screen.getByRole("switch", { name: "Sync" });
  const thumb = control.children[0]?.children[0];
  if (!thumb) throw new Error("switch thumb is missing");
  return thumb.transform?.[4] ?? 0;
};

test("delegates switch thumb movement to a persistent GPUI spring", () => {
  const screen = renderComponent(() => (
    <Switch aria-label="Sync" defaultChecked={false} />
  ));
  const control = screen.getByRole("switch", { name: "Sync" });

  expect(thumbX(screen)).toBe(0);
  control.click();
  expect(control.checked).toBe(true);
  expect(thumbX(screen)).toBe(16);
  const thumb = control.children[0]?.children[0];
  expect(
    JSON.parse(thumb?.attribute("__wabou_native_spring") ?? "null"),
  ).toMatchObject({
    response: 0.16,
    damping: 1,
    epsilon: 0.02,
    targetTransform: [1, 0, 0, 1, 16, 0],
  });
});

test("retargets the same spring instead of replacing the motion primitive", () => {
  const screen = renderComponent(() => <Switch aria-label="Sync" />);
  const control = screen.getByRole("switch", { name: "Sync" });

  control.click();
  const enabledThumb = screen.getByRole("switch", { name: "Sync" }).children[0]
    ?.children[0];
  const enabled = JSON.parse(
    enabledThumb?.attribute("__wabou_native_spring") ?? "null",
  );
  control.click();
  const disabledThumb = screen.getByRole("switch", { name: "Sync" }).children[0]
    ?.children[0];
  const disabled = JSON.parse(
    disabledThumb?.attribute("__wabou_native_spring") ?? "null",
  );

  expect(disabledThumb?.id).toEqual(enabledThumb?.id);
  expect(enabled.targetTransform[4]).toBe(16);
  expect(disabled.targetTransform[4]).toBe(0);
  expect(enabled.response).toBe(disabled.response);
});

test("disables spring travel under reduced motion", () => {
  const screen = renderComponent(() => (
    <MotionConfigProvider reducedMotion>
      <Switch aria-label="Sync" />
    </MotionConfigProvider>
  ));

  screen.getByRole("switch", { name: "Sync" }).click();
  const thumb = screen.getByRole("switch", { name: "Sync" }).children[0]
    ?.children[0];
  expect(
    JSON.parse(thumb?.attribute("__wabou_native_spring") ?? "null"),
  ).toMatchObject({
    response: 0,
    targetTransform: [1, 0, 0, 1, 16, 0],
  });
});

test("publishes the final switch position under reduced motion", () => {
  const screen = renderComponent(() => (
    <MotionConfigProvider reducedMotion>
      <Switch aria-label="Sync" />
    </MotionConfigProvider>
  ));

  screen.getByRole("switch", { name: "Sync" }).click();
  expect(thumbX(screen)).toBe(16);
  const thumb = screen.getByRole("switch", { name: "Sync" }).children[0]
    ?.children[0];
  expect(thumb?.attribute("__wabou_native_transition")).toBeNull();
});

test("uses compact GPUI-native track geometry without focus reflow", () => {
  const screen = renderComponent(() => <Switch aria-label="Sync" />);
  const control = screen.getByRole("switch", { name: "Sync" });
  const track = control.children[0];
  const thumb = track?.children[0];

  expect(control.className).toContain("w-10 h-6");
  expect(control.className).toContain("border border-transparent");
  expect(track?.className).toContain("w-9 h-5");
  expect(track?.className).toContain("overflow-hidden");
  expect(track?.className).not.toContain("border");
  expect(thumb?.className).toContain("w-4 h-4");
  expect(thumb?.className).toContain("bg-surface");
});

test("only keyboard-visible focus paints the focus ring", () => {
  const base = {
    hovered: false,
    pressed: false,
    selected: false,
    disabled: false,
  };
  expect(
    switchControlClass({
      ...base,
      focused: true,
      focusVisible: false,
    }),
  ).not.toContain("border-focus");
  expect(
    switchControlClass({
      ...base,
      focused: true,
      focusVisible: true,
    }),
  ).toContain("border-focus");
});

test("supports gpui-component compact track geometry", () => {
  const screen = renderComponent(() => (
    <Switch aria-label="Compact" size="sm" />
  ));
  const control = screen.getByRole("switch", { name: "Compact" });
  expect(control.className).toContain("w-10 h-6");
  expect(control.children[0]?.className).toContain("w-7 h-4");
  expect(control.children[0]?.children[0]?.className).toContain("w-3 h-3");
});

test("switch label focuses and activates its explicit control", () => {
  const screen = renderComponent(() => (
    <Switch label="Enable subagents" defaultChecked={false} />
  ));
  const control = screen.getByRole("switch", { name: "Enable subagents" });
  const label = screen.getByRole("label", { name: "Enable subagents" });

  label.click();
  expect(control.checked).toBe(true);
  expect(control.focused).toBe(true);
});

test("disabled switch labels cannot activate the control", () => {
  const screen = renderComponent(() => (
    <Switch label="Enable subagents" disabled defaultChecked={false} />
  ));
  const control = screen.getByRole("switch", { name: "Enable subagents" });
  const label = screen.getByRole("label", { name: "Enable subagents" });

  expect(control.className).toContain("cursor-not-allowed");
  expect(control.className).toContain("opacity-60");
  expect(() => label.click()).toThrow("cannot click disabled");
  expect(control.checked).toBe(false);
  expect(control.focused).toBe(false);
});

test("places a start label before the control without splitting activation", () => {
  const screen = renderComponent(() => (
    <Switch
      label="Automatic updates"
      labelPlacement="start"
      defaultChecked={false}
    />
  ));
  const control = screen.getByRole("switch", { name: "Automatic updates" });
  const label = screen.getByRole("label", { name: "Automatic updates" });

  expect(control.parent?.className).toContain("flex-row-reverse");
  expect(control.attribute("labelPlacement")).toBeNull();
  label.click();
  expect(control.checked).toBe(true);
  expect(control.focused).toBe(true);
});
