import AbortControllerPolyfill, {
  AbortSignal as AbortSignalPolyfill,
} from "abort-controller/dist/abort-controller";

/** Install cancellation primitives when the embedding runtime lacks them. */
export function installAbortControllerPolyfill(): void {
  if (!("AbortSignal" in globalThis)) {
    Object.defineProperty(globalThis, "AbortSignal", {
      configurable: true,
      writable: true,
      value: AbortSignalPolyfill,
    });
  }
  if (!("AbortController" in globalThis)) {
    Object.defineProperty(globalThis, "AbortController", {
      configurable: true,
      writable: true,
      value: AbortControllerPolyfill,
    });
  }
}

installAbortControllerPolyfill();
