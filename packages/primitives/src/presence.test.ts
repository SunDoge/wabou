import { expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import { createPresence } from "./presence";

test("presence keeps exiting content mounted until completion", () =>
  createRoot((dispose) => {
    const [open, setOpen] = createSignal(false);
    const presence = createPresence(open);
    expect(presence.phase()).toBe("unmounted");

    setOpen(true);
    flush();
    expect(presence.phase()).toBe("entering");
    expect(presence.mounted()).toBe(true);
    presence.finishEnter();
    flush();
    expect(presence.phase()).toBe("present");

    setOpen(false);
    flush();
    expect(presence.phase()).toBe("exiting");
    expect(presence.mounted()).toBe(true);
    presence.finishExit();
    flush();
    expect(presence.phase()).toBe("unmounted");
    expect(presence.mounted()).toBe(false);
    dispose();
  }));

test("presence coalesces a close and reopen inside one flush", () =>
  createRoot((dispose) => {
    const [open, setOpen] = createSignal(true);
    const presence = createPresence(open);
    setOpen(false);
    setOpen(true);
    flush();
    expect(presence.phase()).toBe("present");
    presence.finishExit();
    flush();
    expect(presence.mounted()).toBe(true);
    presence.finishEnter();
    flush();
    expect(presence.phase()).toBe("present");
    dispose();
  }));
