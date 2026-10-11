import "./dom-exception";
import AbortControllerPolyfill, {
  AbortSignal as AbortSignalPolyfill,
} from "abort-controller/dist/abort-controller";
import { installMissingGlobals } from "./globals";

// Retain the mature EventTarget implementation, adding modern cancellation APIs.
const reasons = new WeakMap<object, unknown>();
const Signal = AbortSignalPolyfill as unknown as typeof AbortSignal;
Object.defineProperties(Signal.prototype, {
  reason: {
    configurable: true,
    get(this: AbortSignal) {
      return reasons.get(this);
    },
  },
  throwIfAborted: {
    configurable: true,
    value(this: AbortSignal) {
      if (this.aborted) throw this.reason;
    },
  },
});
class Controller extends AbortControllerPolyfill {
  abort(
    reason: unknown = new DOMException(
      "The operation was aborted",
      "AbortError",
    ),
  ): void {
    if (this.signal.aborted) return;
    reasons.set(this.signal, reason);
    super.abort();
  }
}
Signal.abort = (reason?: unknown) => {
  const c = new Controller();
  c.abort(reason);
  return c.signal as unknown as AbortSignal;
};
Signal.timeout = (milliseconds: number) => {
  if (!Number.isSafeInteger(milliseconds) || milliseconds < 0)
    throw new RangeError("Invalid timeout");
  const c = new Controller();
  setTimeout(
    () => c.abort(new DOMException("The operation timed out", "TimeoutError")),
    milliseconds,
  );
  return c.signal as unknown as AbortSignal;
};
Signal.any = (signals: AbortSignal[]) => {
  const c = new Controller();
  const cleanups: (() => void)[] = [];
  for (const signal of signals) {
    if (signal.aborted) {
      c.abort(signal.reason);
      break;
    }
    const abort = () => c.abort(signal.reason);
    signal.addEventListener("abort", abort, { once: true });
    cleanups.push(() => signal.removeEventListener("abort", abort));
  }
  const cleanup = () => {
    for (const remove of cleanups) remove();
  };
  if (c.signal.aborted) cleanup();
  else c.signal.addEventListener("abort", cleanup, { once: true });
  return c.signal as unknown as AbortSignal;
};
/** Install cancellation primitives when the embedding runtime lacks them. */
export function installAbortControllerPolyfill(): void {
  installMissingGlobals({ AbortController: Controller, AbortSignal: Signal });
}
installAbortControllerPolyfill();
