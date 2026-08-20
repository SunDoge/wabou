import { renderComponent } from "@wabou/test/component";
import { MotionConfigProvider, Switch } from "@wabou/ui";
import { expect, test } from "vitest";

const thumbX = (screen: ReturnType<typeof renderComponent>): number => {
  const control = screen.getByRole("switch", { name: "Sync" });
  const thumb = control.children[0];
  if (!thumb) throw new Error("switch thumb is missing");
  return thumb.transform?.[4] ?? 0;
};

test("interpolates the switch thumb and settles at the selected position", () => {
  const screen = renderComponent(
    () => <Switch aria-label="Sync" defaultChecked={false} />,
    { clock: "fake" },
  );
  const control = screen.getByRole("switch", { name: "Sync" });

  expect(thumbX(screen)).toBe(0);
  control.click();
  expect(control.checked).toBe(true);
  screen.advanceTime(90);
  expect(thumbX(screen)).toBeGreaterThan(0);
  expect(thumbX(screen)).toBeLessThan(20);
  screen.advanceTime(200);
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
