import { expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import { createTransitionPresence } from "./transition-presence";

test("transition presence waits for readiness and unmounts after exit", () =>
  createRoot((dispose) => {
    const [open, setOpen] = createSignal(false);
    const [ready, setReady] = createSignal(false);
    const presence = createTransitionPresence(open, {
      ready,
      reducedMotion: true,
    });

    expect(presence.mounted()).toBe(false);
    setOpen(true);
    flush();
    expect(presence.phase()).toBe("entering");
    expect(presence.mounted()).toBe(true);
    expect(presence.progress()).toBe(0);

    setReady(true);
    flush();
    expect(presence.phase()).toBe("present");
    expect(presence.progress()).toBe(1);

    setOpen(false);
    flush();
    expect(presence.phase()).toBe("unmounted");
    expect(presence.progress()).toBe(0);
    dispose();
  }));

test("transition presence coalesces a reopen before exit advances", () =>
  createRoot((dispose) => {
    const [open, setOpen] = createSignal(true);
    const presence = createTransitionPresence(open, {
      duration: 10,
    });

    expect(presence.phase()).toBe("present");
    setOpen(false);
    flush();
    expect(presence.phase()).toBe("exiting");
    expect(presence.mounted()).toBe(true);
    setOpen(true);
    flush();
    expect(presence.phase()).toBe("present");
    expect(presence.mounted()).toBe(true);
    dispose();
  }));
