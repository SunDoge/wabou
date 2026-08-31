import { renderComponent } from "@wabou/test/component";
import { MotionConfigProvider, Spinner } from "@wabou/ui";
import { expect, test } from "vitest";

test("declares a native loop instead of publishing per-frame transforms", () => {
  const screen = renderComponent(() => (
    <Spinner label="Synchronizing" duration={1.25} speed={1.5} />
  ));
  const spinner = screen.getByRole("status", { name: "Synchronizing" });

  expect(spinner.tag).toBe("spinner");
  expect(spinner.transform).toBeNull();
  expect(spinner.widgetConfig).toEqual({
    animation: {
      kind: "loop",
      duration: 1.25,
      speed: 1.5,
      paused: false,
      reducedMotion: false,
    },
  });
});

test("forwards reduced-motion policy to the native executor", () => {
  const screen = renderComponent(() => (
    <MotionConfigProvider reducedMotion>
      <Spinner />
    </MotionConfigProvider>
  ));

  expect(screen.getByRole("status").widgetConfig).toEqual({
    animation: {
      kind: "loop",
      duration: 0.9,
      speed: 1,
      paused: false,
      reducedMotion: true,
    },
  });
});
