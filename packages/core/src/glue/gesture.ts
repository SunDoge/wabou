import { onCleanup } from "solid-js";
import { subscribeJson } from "./host-messages";

export type GesturePhase = "started" | "changed" | "ended" | "cancelled";

export type GestureEvent =
  | { type: "pinch"; delta: number; phase: GesturePhase }
  | { type: "pan"; deltaX: number; deltaY: number; phase: GesturePhase }
  | { type: "rotation"; delta: number; phase: GesturePhase }
  | { type: "double-tap" }
  | { type: "pressure"; pressure: number; stage: number };

export type GestureHandler = (event: GestureEvent) => void;

function isPhase(value: unknown): value is GesturePhase {
  return (
    value === "started" ||
    value === "changed" ||
    value === "ended" ||
    value === "cancelled"
  );
}

function decodeGesture(value: unknown): GestureEvent {
  if (typeof value !== "object" || value === null)
    throw new TypeError("gesture event must be an object");
  const event = value as Record<string, unknown>;
  if (event.type === "double-tap") return { type: "double-tap" };
  if (
    event.type === "pressure" &&
    typeof event.pressure === "number" &&
    typeof event.stage === "number"
  ) {
    return { type: "pressure", pressure: event.pressure, stage: event.stage };
  }
  if (!isPhase(event.phase))
    throw new TypeError("continuous gesture event has an invalid phase");
  if (
    (event.type === "pinch" || event.type === "rotation") &&
    typeof event.delta === "number"
  ) {
    return { type: event.type, delta: event.delta, phase: event.phase };
  }
  if (
    event.type === "pan" &&
    typeof event.deltaX === "number" &&
    typeof event.deltaY === "number"
  ) {
    return {
      type: "pan",
      deltaX: event.deltaX,
      deltaY: event.deltaY,
      phase: event.phase,
    };
  }
  throw new TypeError("gesture event has an invalid payload");
}

/** Subscribe to native trackpad and touchscreen gestures for the current window. */
export function subscribeGesture(handler: GestureHandler): () => void {
  return subscribeJson("wabou:gesture", handler, { decode: decodeGesture });
}

/** Subscribe for the lifetime of the current Solid owner. */
export function useGesture(handler: GestureHandler): void {
  onCleanup(subscribeGesture(handler));
}
