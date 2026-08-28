import { onCleanup } from "solid-js";
import { subscribeJson } from "./host-messages";

export type AppLifecycleState = "resumed" | "suspended" | "memory-warning";

export interface AppLifecycleEvent {
  state: AppLifecycleState;
}

function decodeAppLifecycle(value: unknown): AppLifecycleEvent {
  if (typeof value !== "object" || value === null)
    throw new TypeError("application lifecycle event must be an object");
  const state = (value as { state?: unknown }).state;
  if (
    state !== "resumed" &&
    state !== "suspended" &&
    state !== "memory-warning"
  ) {
    throw new TypeError("application lifecycle event has an invalid state");
  }
  return { state };
}

/** Subscribe to operating-system lifecycle notifications. */
export function subscribeAppLifecycle(
  handler: (event: AppLifecycleEvent) => void,
): () => void {
  return subscribeJson("wabou:app-lifecycle", handler, {
    decode: decodeAppLifecycle,
  });
}

/** Subscribe for the lifetime of the current Solid owner. */
export function useAppLifecycle(
  handler: (event: AppLifecycleEvent) => void,
): void {
  onCleanup(subscribeAppLifecycle(handler));
}
