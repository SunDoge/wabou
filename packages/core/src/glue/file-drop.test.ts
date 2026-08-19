import { createRoot } from "solid-js";
import { describe, expect, test, vi } from "vitest";
import { subscribeFileDrop, useFileDrop } from "./file-drop";
import { dispatchHostMessage } from "./host-messages";

describe("file drop", () => {
  test("decodes explicit native file phases", () => {
    const handler = vi.fn();
    const unsubscribe = subscribeFileDrop(handler);

    dispatchHostMessage(
      "wabou:file-drop",
      JSON.stringify({
        phase: "entered",
        paths: ["/tmp/demo.yaml"],
        position: { x: 20, y: 30 },
      }),
    );
    dispatchHostMessage(
      "wabou:file-drop",
      JSON.stringify({ phase: "left", paths: [], position: null }),
    );
    dispatchHostMessage(
      "wabou:file-drop",
      JSON.stringify({
        phase: "dropped",
        paths: ["/tmp/demo.yaml"],
        position: { x: 21, y: 31 },
      }),
    );

    expect(handler.mock.calls.map(([event]) => event)).toEqual([
      {
        phase: "entered",
        paths: ["/tmp/demo.yaml"],
        position: { x: 20, y: 30 },
      },
      { phase: "left", paths: [], position: null },
      {
        phase: "dropped",
        paths: ["/tmp/demo.yaml"],
        position: { x: 21, y: 31 },
      },
    ]);
    unsubscribe();
  });

  test("Solid helper unsubscribes with its owner", () => {
    const handler = vi.fn();
    const dispose = createRoot((dispose) => {
      useFileDrop(handler);
      return dispose;
    });
    dispose();

    dispatchHostMessage(
      "wabou:file-drop",
      JSON.stringify({
        phase: "dropped",
        paths: ["/tmp/ignored"],
        position: null,
      }),
    );
    expect(handler).not.toHaveBeenCalled();
  });
});
