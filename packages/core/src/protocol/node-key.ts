const U32_MAX = 0xffff_ffff;

declare const nodeKeyBrand: unique symbol;

/**
 * Full-width retained-node identity used on both sides of the native bridge.
 * `lo` selects a slot and `hi` identifies that slot's generation.
 */
export interface NodeKey {
  readonly lo: number;
  readonly hi: number;
  readonly [nodeKeyBrand]: "NodeKey";
}

/** Structural form accepted when a key was deserialized from JSON. */
export type NodeKeyParts = Pick<NodeKey, "lo" | "hi">;

export const ROOT_NODE_KEY: NodeKey = nodeKey(1, 1);

function u32(value: number, field: string): number {
  if (!Number.isInteger(value) || value < 0 || value > U32_MAX) {
    throw new RangeError(`${field} must be an unsigned 32-bit integer`);
  }
  return value;
}

/** Construct a node key received from a trusted binary boundary. */
export function nodeKey(lo: number, hi: number): NodeKey {
  lo = u32(lo, "NodeKey.lo");
  hi = u32(hi, "NodeKey.hi");
  if (lo === 0) throw new RangeError("NodeKey slot zero is reserved");
  if ((hi & 1) === 0) {
    throw new RangeError("NodeKey generation must be a non-zero odd u32");
  }
  return { lo, hi } as NodeKey;
}

export function isNodeKey(value: unknown): value is NodeKey {
  if (value === null || typeof value !== "object") return false;
  const candidate = value as Partial<NodeKey>;
  const { lo, hi } = candidate;
  return (
    typeof lo === "number" &&
    Number.isInteger(lo) &&
    lo > 0 &&
    lo <= U32_MAX &&
    typeof hi === "number" &&
    Number.isInteger(hi) &&
    hi > 0 &&
    hi <= U32_MAX &&
    (hi & 1) === 1
  );
}

export function nodeKeyEquals(
  left: NodeKey | null | undefined,
  right: NodeKey | null | undefined,
): boolean {
  return (
    left === right ||
    (!!left && !!right && left.lo === right.lo && left.hi === right.hi)
  );
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
    this.#nextSlot = u32(firstSlot, "firstSlot");
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
    if (next <= U32_MAX) {
      this.#generations[key.lo] = next;
      this.#free.push(key.lo);
    }
    return true;
  }

  isLive(key: NodeKey): boolean {
    return this.#live[key.lo] === true && this.#generations[key.lo] === key.hi;
  }

  #allocateSlot(): number {
    if (this.#nextSlot > U32_MAX) {
      throw new RangeError("NodeKey slot space exhausted");
    }
    return this.#nextSlot++;
  }
}

/**
 * Slot-indexed storage which always validates the complete generational key.
 * This keeps array lookup speed without allowing stale-key aliasing.
 */
export class NodeKeyTable<T> {
  readonly #entries: ({ hi: number; value: T } | undefined)[] = [];

  set(key: NodeKey, value: T): this {
    this.#entries[key.lo] = { hi: key.hi, value };
    return this;
  }

  get(key: NodeKey): T | undefined {
    const entry = this.#entries[key.lo];
    return entry?.hi === key.hi ? entry.value : undefined;
  }

  has(key: NodeKey): boolean {
    return this.#entries[key.lo]?.hi === key.hi;
  }

  delete(key: NodeKey): boolean {
    const entry = this.#entries[key.lo];
    if (entry?.hi !== key.hi) return false;
    this.#entries[key.lo] = undefined;
    return true;
  }

  clear(): void {
    this.#entries.length = 0;
  }
}
