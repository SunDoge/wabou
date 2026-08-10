import { expect, test } from "bun:test";
import type { Handle } from "@wabou/solid-renderer";
import { createRoot } from "solid-js";
import { createMeasuredSize } from "./measure";

test("measured size follows ResizeObserver and disconnects with its owner", () => {
  const OriginalResizeObserver = globalThis.ResizeObserver;
  let callback: ResizeObserverCallback | undefined;
  let observed: unknown;
  let disconnects = 0;
  globalThis.ResizeObserver = class {
    constructor(next: ResizeObserverCallback) {
      callback = next;
    }
    observe(target: Element) {
      observed = target;
    }
    unobserve() {}
    disconnect() {
      disconnects++;
    }
  } as unknown as typeof ResizeObserver;

  try {
    createRoot((dispose) => {
      const size = createMeasuredSize();
      const node = { id: 42 } as Handle;
      size.ref(node);
      expect(observed).toBe(node);
      callback?.(
        [
          {
            target: node,
            contentRect: { width: 120, height: 48 },
          } as unknown as ResizeObserverEntry,
        ],
        {} as ResizeObserver,
      );
      expect(size.measured()).toBe(true);
      expect(size.width()).toBe(120);
      expect(size.height()).toBe(48);
      dispose();
      expect(disconnects).toBe(1);
    });
  } finally {
    globalThis.ResizeObserver = OriginalResizeObserver;
  }
});
