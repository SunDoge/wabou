// requestAnimationFrame host glue. The host (wabou-shell) drives rendering;
// `__wabou_tick` is called once per frame to drain queued rAF callbacks, run the
// solid-renderer sweep (finalization-registry cleanup), and flush the binary
// protocol frame back to Rust. `__wabou_has_raf` tells the host whether to keep
// redrawing.

import { flush } from "solid-js";
import { runSweep, writer } from "../renderer";

export class AnimationFrameQueue {
  readonly #callbacks = new Map<number, (time: number) => void>();
  #nextId = 1;

  request(callback: (time: number) => void): number {
    const id = this.#nextId++;
    this.#callbacks.set(id, callback);
    return id;
  }

  cancel(id: number): void {
    this.#callbacks.delete(id);
  }

  drain(): Array<[number, (time: number) => void]> {
    const entries = Array.from(this.#callbacks.entries());
    this.#callbacks.clear();
    return entries;
  }

  hasPending(): boolean {
    return this.#callbacks.size > 0;
  }
}

const animationFrames = new AnimationFrameQueue();

export function requestAnimationFrameImpl(cb: (t: number) => void): number {
  return animationFrames.request(cb);
}

function cancelAnimationFrameImpl(id: number): void {
  animationFrames.cancel(id);
}

export function tickAnimationFrame(
  frameTime: number,
  deliver: (bytes: Uint8Array) => void = __wabou_flush,
  flushWriter: () => Uint8Array | null | undefined = () => writer.flush(),
  commit: <T>(callback: () => T) => T = flush,
  queue: AnimationFrameQueue = animationFrames,
): boolean {
  const entries = queue.drain();
  // A native frame is the transaction boundary for every rAF callback.
  // Commit Solid's queued render effects before serializing the writer, so
  // rAF-driven changes cannot sit in the writer until an unrelated next frame.
  commit(() => {
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
  return queue.hasPending();
}

function __wabou_tick(frameTime: number): boolean {
  return tickAnimationFrame(frameTime);
}

function __wabou_has_raf(): boolean {
  return animationFrames.hasPending();
}

(globalThis as unknown as Record<string, unknown>).requestAnimationFrame =
  requestAnimationFrameImpl;
(globalThis as unknown as Record<string, unknown>).cancelAnimationFrame =
  cancelAnimationFrameImpl;
(globalThis as unknown as Record<string, unknown>).__wabou_tick = __wabou_tick;
(globalThis as unknown as Record<string, unknown>).__wabou_has_raf =
  __wabou_has_raf;
