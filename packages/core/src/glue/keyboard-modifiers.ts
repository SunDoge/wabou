import { type Accessor, createSignal, onCleanup } from "solid-js";
import { subscribe } from "./host-messages";

export interface KeyboardModifiers {
  readonly bits: number;
  readonly shift: boolean;
  readonly control: boolean;
  readonly alt: boolean;
  readonly meta: boolean;
  readonly primary: boolean;
}

const SHIFT = 1 << 0;
const CONTROL = 1 << 1;
const ALT = 1 << 2;
const META = 1 << 3;
const PRIMARY = 1 << 4;
const VALID_MASK = SHIFT | CONTROL | ALT | META | PRIMARY;

function decodeKeyboardModifiers(value: unknown): KeyboardModifiers {
  if (
    !Number.isInteger(value) ||
    (value as number) < 0 ||
    ((value as number) & ~VALID_MASK) !== 0
  ) {
    throw new TypeError("keyboard modifier bits are invalid");
  }
  const bits = value as number;
  return Object.freeze({
    bits: bits & (SHIFT | CONTROL | ALT | META),
    shift: (bits & SHIFT) !== 0,
    control: (bits & CONTROL) !== 0,
    alt: (bits & ALT) !== 0,
    meta: (bits & META) !== 0,
    primary: (bits & PRIMARY) !== 0,
  });
}

const empty = decodeKeyboardModifiers(0);
const [keyboardModifiers, setKeyboardModifiers] = createSignal(empty, {
  equals: (previous, next) =>
    previous.bits === next.bits && previous.primary === next.primary,
  ownedWrite: true,
});

const subscribers = new Set<(modifiers: KeyboardModifiers) => void>();

subscribe("wabou:keyboard-modifiers", (payload) => {
  try {
    const modifiers = decodeKeyboardModifiers(payload);
    setKeyboardModifiers(modifiers);
    for (const subscriber of subscribers) {
      try {
        subscriber(modifiers);
      } catch (error) {
        console.error("[wabou-host] keyboard modifier subscriber threw", error);
      }
    }
  } catch (error) {
    console.error("[wabou-host] invalid keyboard modifiers", error);
  }
});

/** Reactive, Host-authoritative physical modifier-key state. */
export function useKeyboardModifiers(): Accessor<KeyboardModifiers> {
  return keyboardModifiers;
}

/** Subscribe to physical modifier changes without creating a Solid owner. */
export function subscribeKeyboardModifiers(
  handler: (modifiers: KeyboardModifiers) => void,
): () => void {
  subscribers.add(handler);
  return () => subscribers.delete(handler);
}

/** Subscribe for the lifetime of the current Solid owner. */
export function useKeyboardModifierChanges(
  handler: (modifiers: KeyboardModifiers) => void,
): void {
  onCleanup(subscribeKeyboardModifiers(handler));
}
