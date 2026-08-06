// requestAnimationFrame host glue. The host (wabou-shell) drives rendering;
// `__wabou_tick` is called once per frame to drain queued rAF callbacks, run the
// solid-renderer sweep (finalization-registry cleanup), and flush the binary
// protocol frame back to Rust. `__wabou_has_raf` tells the host whether to keep
// redrawing.

import { runSweep, writer } from "@wabou/solid-renderer";

const rafQueue = new Map<number, (t: number) => void>();
let nextRafId = 1;

function requestAnimationFrameImpl(cb: (t: number) => void): number {
  const id = nextRafId++;
  rafQueue.set(id, cb);
  return id;
}

function cancelAnimationFrameImpl(id: number): void {
  rafQueue.delete(id);
}

function __wabou_tick(frameTime: number): boolean {
  const entries = Array.from(rafQueue.entries());
  rafQueue.clear();
  for (const [_, cb] of entries) {
    try {
      cb(frameTime);
    } catch (e: any) {
      __wabou_log("error", e.stack ? String(e.stack) : String(e));
    }
  }
  runSweep();
  const bytes = writer.flush();
  if (bytes) __wabou_flush(bytes);
  return rafQueue.size > 0;
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
