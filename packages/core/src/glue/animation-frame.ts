// requestAnimationFrame host glue. The host (wabou-shell) drives rendering;
// `__wabou_tick` is called once per frame to drain queued rAF callbacks, run the
// solid-renderer sweep (finalization-registry cleanup), and flush the binary
// protocol frame back to Rust. `__wabou_has_raf` tells the host whether to keep
// redrawing.

import { flush } from "solid-js";
import { runSweep, writer } from "../renderer";

const rafQueue = new Map<number, (t: number) => void>();
let nextRafId = 1;

export function requestAnimationFrameImpl(cb: (t: number) => void): number {
  const id = nextRafId++;
  rafQueue.set(id, cb);
  return id;
}

function cancelAnimationFrameImpl(id: number): void {
  rafQueue.delete(id);
}

export function tickAnimationFrame(
  frameTime: number,
  deliver: (bytes: Uint8Array) => void = __wabou_flush,
  flushWriter: () => Uint8Array | null | undefined = () => writer.flush(),
): boolean {
  const entries = Array.from(rafQueue.entries());
  rafQueue.clear();
  // A native frame is the transaction boundary for every rAF callback.
  // Commit Solid's queued render effects before serializing the writer, so
  // rAF-driven changes cannot sit in the writer until an unrelated next frame.
  flush(() => {
    for (const [_, cb] of entries) {
      try {
        cb(frameTime);
      } catch (error: unknown) {
        __wabou_log(
          "error",
          error instanceof Error && error.stack ? error.stack : String(error),
        );
      }
    }
  });
  runSweep();
  const bytes = flushWriter();
  if (bytes) deliver(bytes);
  return rafQueue.size > 0;
}

function __wabou_tick(frameTime: number): boolean {
  return tickAnimationFrame(frameTime);
}

function __wabou_has_raf(): boolean {
  return rafQueue.size > 0;
}

(globalThis as unknown as Record<string, unknown>).requestAnimationFrame =
  requestAnimationFrameImpl;
(globalThis as unknown as Record<string, unknown>).cancelAnimationFrame =
  cancelAnimationFrameImpl;
(globalThis as unknown as Record<string, unknown>).__wabou_tick = __wabou_tick;
(globalThis as unknown as Record<string, unknown>).__wabou_has_raf =
  __wabou_has_raf;
