import { expect, test } from "bun:test";
import { nodeKey } from "@wabou/core/protocol";
import type { Handle } from "@wabou/core/renderer";
import { createRoot, flush } from "solid-js";
import { createContainerMatch, createMeasuredSize } from "./measure";

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
      const node = { id: nodeKey(42, 1) } as unknown as Handle;
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
      flush();
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

test("container match follows its own measured content box", () => {
  const OriginalResizeObserver = globalThis.ResizeObserver;
  let callback: ResizeObserverCallback | undefined;
  globalThis.ResizeObserver = class {
    constructor(next: ResizeObserverCallback) {
      callback = next;
    }
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;

  try {
    createRoot((dispose) => {
      const query = createContainerMatch({ minWidth: 400, maxWidth: 700 });
      const node = { id: nodeKey(43, 1) } as unknown as Handle;
      query.ref(node);
      expect(query.measured()).toBe(false);
      expect(query.matches()).toBe(false);

      const resize = (width: number, height = 80) => {
        callback?.(
          [
            {
              target: node,
              contentRect: { width, height },
            } as unknown as ResizeObserverEntry,
          ],
          {} as ResizeObserver,
        );
        flush();
      };
      resize(360);
      expect(query.matches()).toBe(false);
      resize(620);
      expect(query.matches()).toBe(true);
      resize(720);
      expect(query.matches()).toBe(false);
      dispose();
    });
  } finally {
    globalThis.ResizeObserver = OriginalResizeObserver;
  }
});

test("container match rejects contradictory and non-finite constraints", () => {
  expect(() => createContainerMatch({ minWidth: 10, maxWidth: 9 })).toThrow(
    "minWidth cannot exceed maxWidth",
  );
  expect(() => createContainerMatch({ maxHeight: Number.NaN })).toThrow(
    "maxHeight must be a finite non-negative number",
  );
});
