import { renderComponent } from "@wabou/test/component";
import { MotionConfigProvider, Switch } from "@wabou/ui";
import { expect, test } from "vitest";

const thumbX = (screen: ReturnType<typeof renderComponent>): number => {
  const control = screen.getByRole("switch", { name: "Sync" });
  const thumb = control.children[0];
  if (!thumb) throw new Error("switch thumb is missing");
  return thumb.transform?.[4] ?? 0;
};

test("interpolates the switch thumb and settles at the selected position", async () => {
  const screen = renderComponent(
    () => <Switch aria-label="Sync" defaultChecked={false} />,
    { clock: "fake" },
  );
  const control = screen.getByRole("switch", { name: "Sync" });

  expect(thumbX(screen)).toBe(0);
  control.click();
  expect(control.checked).toBe(true);
  await screen.advanceTime(90);
  expect(thumbX(screen)).toBeGreaterThan(0);
  expect(thumbX(screen)).toBeLessThan(20);
  await screen.advanceTime(200);
  expect(thumbX(screen)).toBe(20);
});

test("publishes the final switch position under reduced motion", () => {
  const screen = renderComponent(() => (
    <MotionConfigProvider reducedMotion>
      <Switch aria-label="Sync" />
    </MotionConfigProvider>
  ));

  screen.getByRole("switch", { name: "Sync" }).click();
  expect(thumbX(screen)).toBe(20);
});

test("uses compact GPUI-native track geometry without focus reflow", () => {
  const screen = renderComponent(() => <Switch aria-label="Sync" />);
  const control = screen.getByRole("switch", { name: "Sync" });
  const thumb = control.children[0];

  expect(control.className).toContain("w-10 h-6");
  expect(control.className).toContain("overflow-hidden");
  expect(control.className).toContain("border border-transparent");
  expect(thumb?.className).toContain("w-4 h-4");
  expect(thumb?.className).toContain("bg-surface");
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

  expect(() => label.click()).toThrow("cannot click disabled");
  expect(control.checked).toBe(false);
  expect(control.focused).toBe(false);
});
