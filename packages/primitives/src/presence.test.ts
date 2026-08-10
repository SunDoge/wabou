import { expect, test } from "bun:test";
import { createRoot, createSignal } from "solid-js";
import { createPresence } from "./presence";

test("presence keeps exiting content mounted until completion", () =>
  createRoot((dispose) => {
    const [open, setOpen] = createSignal(false);
    const presence = createPresence(open);
    expect(presence.phase()).toBe("unmounted");

    setOpen(true);
    expect(presence.phase()).toBe("entering");
    expect(presence.mounted()).toBe(true);
    presence.finishEnter();
    expect(presence.phase()).toBe("present");

    setOpen(false);
    expect(presence.phase()).toBe("exiting");
    expect(presence.mounted()).toBe(true);
    presence.finishExit();
    expect(presence.phase()).toBe("unmounted");
    expect(presence.mounted()).toBe(false);
    dispose();
  }));

test("presence ignores stale completion after a rapid reopen", () =>
  createRoot((dispose) => {
    const [open, setOpen] = createSignal(true);
    const presence = createPresence(open);
    setOpen(false);
    setOpen(true);
    expect(presence.phase()).toBe("entering");
    presence.finishExit();
    expect(presence.mounted()).toBe(true);
    presence.finishEnter();
    expect(presence.phase()).toBe("present");
    dispose();
  }));
