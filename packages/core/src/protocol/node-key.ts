import {
  createResourceKeyFamily,
  isResourceKeyParts,
  type ResourceKey,
  type ResourceKeyParts,
  ResourceKeyTable,
} from "./resource-key";

/**
 * Full-width retained-node identity used on both sides of the native bridge.
 * `lo` selects a slot and `hi` identifies that slot's generation.
 */
export type NodeKey = ResourceKey<"node">;

/** Structural form accepted when a key was deserialized from JSON. */
export type NodeKeyParts = ResourceKeyParts;

// Node allocation is a hot path and nodes have only one identity family.
// Independent Rust-owned resources keep the default runtime family token.
const nodeKeyFamily = createResourceKeyFamily("node", { runtimeBrand: false });

export const ROOT_NODE_KEY: NodeKey = nodeKey(1, 1);

/** Construct a node key received from a trusted binary boundary. */
export function nodeKey(lo: number, hi: number): NodeKey {
  return nodeKeyFamily.fromParts(lo, hi);
}

export function isNodeKey(value: unknown): value is NodeKey {
  // Node keys also arrive as plain `{ lo, hi }` records from JSON diagnostics.
  // Resource families use their stricter family-bound `is` method when
  // distinguishing independently owned resource types matters.
  return isResourceKeyParts(value);
}

export function nodeKeyEquals(
  left: NodeKey | null | undefined,
  right: NodeKey | null | undefined,
): boolean {
  return nodeKeyFamily.equals(left, right);
}

/** Stable diagnostic form; do not use it on the binary hot path. */
export function formatNodeKey(key: NodeKeyParts): string {
  return `${key.lo}v${key.hi}`;
}

/**
 * Splits SlotMap's `KeyData::as_ffi()` representation without converting the
 * full value to an imprecise JavaScript number.
 */
export function nodeKeyFromSlotMapFfi(lo: number, hi: number): NodeKey {
  return nodeKey(lo, hi);
}

/**
 * Allocates full-width generational node keys. Exhausted generations retire a
 * slot instead of wrapping and making a stale key valid again.
 */
export class NodeKeyAllocator {
  readonly #generations: number[] = [];
  readonly #live: boolean[] = [];
  readonly #free: number[] = [];
  #nextSlot: number;

  constructor(firstSlot = 2) {
    if (!Number.isInteger(firstSlot) || firstSlot < 0 || firstSlot > 0xffff_ffff) {
      throw new RangeError("firstSlot must be an unsigned 32-bit integer");
    }
    this.#nextSlot = firstSlot;
    if (firstSlot === 0) throw new RangeError("slot zero is reserved");
  }

  allocate(): NodeKey {
    const recycled = this.#free.pop();
    const lo = recycled ?? this.#allocateSlot();
    const hi = this.#generations[lo] ?? 1;
    this.#generations[lo] = hi;
    this.#live[lo] = true;
    return nodeKey(lo, hi);
  }

  release(key: NodeKey): boolean {
    if (!this.isLive(key)) return false;
    this.#live[key.lo] = false;
    const next = key.hi + 2;
    if (next <= 0xffff_ffff) {
      this.#generations[key.lo] = next;
      this.#free.push(key.lo);
    }
    return true;
  }

  isLive(key: NodeKey): boolean {
    return this.#live[key.lo] === true && this.#generations[key.lo] === key.hi;
  }

  #allocateSlot(): number {
    if (this.#nextSlot > 0xffff_ffff) {
      throw new RangeError("NodeKey slot space exhausted");
    }
    return this.#nextSlot++;
  }
}

/**
 * Slot-indexed storage which always validates the complete generational key.
 * This keeps array lookup speed without allowing stale-key aliasing.
 */
export class NodeKeyTable<T> extends ResourceKeyTable<"node", T> {
  constructor() {
    super(nodeKeyFamily);
  }
}
