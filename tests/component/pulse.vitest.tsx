import { PlatformProvider, useWindow } from "@wabou/core";
import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import {
  PulseOverview,
  PulseWindowControls,
} from "../../apps/gallery/ui/pulse/overview";

test("Pulse interval and pause controls keep their state without replacing the metrics", () => {
  const screen = renderComponent(() => <PulseOverview />);
  const cpu = screen.getByRole("group", { name: "CPU metric" });
  const five = screen.getByRole("button", { name: "History 5m" });
  five.click();
  expect(five.attribute("aria-pressed")).toBe("true");
  expect(
    screen
      .getByRole("button", { name: "History 1m" })
      .attribute("aria-pressed"),
  ).toBe("false");
  expect(screen.getByRole("group", { name: "CPU metric" }).identity).toEqual(
    cpu.identity,
  );
  screen.getByRole("button", { name: "Pause sample history" }).click();
  expect(
    screen
      .getByRole("button", { name: "Resume sample history" })
      .attribute("aria-pressed"),
  ).toBe("true");
  screen.getByRole("button", { name: "Resume sample history" }).click();
  expect(
    screen
      .getByRole("button", { name: "Pause sample history" })
      .attribute("aria-pressed"),
  ).toBe("false");
});

test("Pulse window buttons invoke native controls and do not start a drag", () => {
  const close = vi.fn();
  const minimize = vi.fn();
  const maximize = vi.fn();
  const drag = vi.fn();
  const screen = renderComponent(() => {
    const native = useWindow();
    const services = {
      window: Object.assign(Object.create(native), {
        close,
        minimize,
        setMaximized: maximize,
        startDragging: drag,
      }),
    };
    return (
      <PlatformProvider value={services}>
        <PulseWindowControls />
      </PlatformProvider>
    );
  });
  screen.getByRole("button", { name: "Minimize window" }).click();
  expect(minimize).toHaveBeenCalledOnce();
  screen.getByRole("button", { name: "Toggle window maximized" }).click();
  expect(maximize).toHaveBeenCalledWith(true);
  screen.getByRole("button", { name: "Close window" }).click();
  expect(close).toHaveBeenCalledOnce();
  expect(drag).not.toHaveBeenCalled();
  screen.getByRole("group", { name: "Move Pulse window" }).pointerDown();
  expect(drag).toHaveBeenCalledOnce();
});
