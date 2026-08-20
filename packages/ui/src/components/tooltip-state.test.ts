import { afterEach, expect, test } from "bun:test";
import { createTooltipDelayController } from "./tooltip-state";

const timers = new Set<ReturnType<typeof setTimeout>>();

afterEach(() => {
  for (const timer of timers) clearTimeout(timer);
  timers.clear();
});

const wait = (duration: number) =>
  new Promise<void>((resolve) => {
    const timer = setTimeout(() => {
      timers.delete(timer);
      resolve();
    }, duration);
    timers.add(timer);
  });

test("cancels a pending open when the pointer leaves", async () => {
  const states: boolean[] = [];
  const controller = createTooltipDelayController({
    openDelay: () => 10,
    closeDelay: () => 0,
    setOpen: (open) => states.push(open),
  });
  controller.scheduleOpen();
  controller.scheduleClose();
  await wait(20);
  expect(states).toEqual([false]);
});

test("keyboard focus opens immediately and dispose cancels work", async () => {
  const states: boolean[] = [];
  const controller = createTooltipDelayController({
    openDelay: () => 10,
    closeDelay: () => 10,
    setOpen: (open) => states.push(open),
  });
  controller.openNow();
  controller.scheduleClose();
  controller.dispose();
  await wait(20);
  expect(states).toEqual([true]);
});
