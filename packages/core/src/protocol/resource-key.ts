const U32_MAX = 0xffff_ffff;

declare const resourceKeyBrand: unique symbol;
const resourceKeyFamily = Symbol("wabou.resource-key-family");

/** Two-u32 representation shared by SlotMap-backed resource handles. */
export interface ResourceKeyParts {
  readonly lo: number;
  readonly hi: number;
}

/**
 * Opaque generational resource identity. `Family` prevents image, font,
 * subscription, and other independently owned resources from being mixed.
 */
export interface ResourceKey<Family extends string> extends ResourceKeyParts {
  readonly [resourceKeyBrand]: Family;
}

type RuntimeResourceKey = ResourceKeyParts & {
  readonly [resourceKeyFamily]?: symbol;
};

function u32(value: number, field: string): number {
  if (!Number.isInteger(value) || value < 0 || value > U32_MAX) {
    throw new RangeError(`${field} must be an unsigned 32-bit integer`);
  }
  return value;
}

/** Validate the common two-u32 SlotMap wire representation. */
export function validateResourceKeyParts(
  value: ResourceKeyParts,
  label = "ResourceKey",
): ResourceKeyParts {
  const lo = u32(value.lo, `${label}.lo`);
  const hi = u32(value.hi, `${label}.hi`);
  if (lo === 0) throw new RangeError(`${label} slot zero is reserved`);
  if ((hi & 1) === 0) {
    throw new RangeError(`${label} generation must be a non-zero odd u32`);
  }
  return { lo, hi };
}

/** Structural check for a key arriving through JSON or another untyped edge. */
export function isResourceKeyParts(value: unknown): value is ResourceKeyParts {
  if (value === null || typeof value !== "object") return false;
  const candidate = value as Partial<ResourceKeyParts>;
  return (
    typeof candidate.lo === "number" &&
    Number.isInteger(candidate.lo) &&
    candidate.lo > 0 &&
    candidate.lo <= U32_MAX &&
    typeof candidate.hi === "number" &&
    Number.isInteger(candidate.hi) &&
    candidate.hi > 0 &&
    candidate.hi <= U32_MAX &&
    (candidate.hi & 1) === 1
  );
}

/** Stable diagnostic form; binary paths continue to write two u32 fields. */
export function formatResourceKeyParts(value: ResourceKeyParts): string {
  return `${value.lo}v${value.hi}`;
}

/** Slot-indexed storage that validates both the family and generation. */
export class ResourceKeyTable<Family extends string, Value> {
  readonly #family: ResourceKeyFamily<Family>;
  readonly #entries: ({ hi: number; value: Value } | undefined)[] = [];

  constructor(family: ResourceKeyFamily<Family>) {
    this.#family = family;
  }

  set(key: ResourceKey<Family>, value: Value): this {
    this.#family.assert(key);
    this.#entries[key.lo] = { hi: key.hi, value };
    return this;
  }

  get(key: ResourceKey<Family>): Value | undefined {
    if (!this.#family.is(key)) return undefined;
    const entry = this.#entries[key.lo];
    return entry?.hi === key.hi ? entry.value : undefined;
  }

  has(key: ResourceKey<Family>): boolean {
    return this.#family.is(key) && this.#entries[key.lo]?.hi === key.hi;
  }

  delete(key: ResourceKey<Family>): boolean {
    if (!this.#family.is(key)) return false;
    const entry = this.#entries[key.lo];
    if (entry?.hi !== key.hi) return false;
    this.#entries[key.lo] = undefined;
    return true;
  }

  clear(): void {
    this.#entries.length = 0;
  }
}

/** Operations bound to one resource family and its private runtime token. */
export interface ResourceKeyFamily<Family extends string> {
  readonly name: Family;
  fromParts(lo: number, hi: number): ResourceKey<Family>;
  fromJSON(value: unknown): ResourceKey<Family>;
  is(value: unknown): value is ResourceKey<Family>;
  assert(value: unknown): asserts value is ResourceKey<Family>;
  equals(
    left: ResourceKey<Family> | null | undefined,
    right: ResourceKey<Family> | null | undefined,
  ): boolean;
  format(value: ResourceKeyParts): string;
  table<Value>(): ResourceKeyTable<Family, Value>;
}

/**
 * Define one opaque handle family. The private symbol token also catches
 * accidental cross-family casts at runtime; it is not serialized on the wire.
 */
export function createResourceKeyFamily<const Family extends string>(
  name: Family,
  options: { readonly runtimeBrand?: boolean } = {},
): ResourceKeyFamily<Family> {
  const token = Symbol(`wabou.resource-key.${name}`);
  const runtimeBrand = options.runtimeBrand ?? true;
  const fromParts = (lo: number, hi: number): ResourceKey<Family> => {
    const parts = validateResourceKeyParts({ lo, hi }, `${name} key`);
    // Keep the family token out of spreads, snapshots, JSON, and public
    // structural equality while retaining a cheap runtime family check.
    if (runtimeBrand) {
      Object.defineProperty(parts, resourceKeyFamily, { value: token });
    }
    return parts as unknown as ResourceKey<Family>;
  };
  const is = (value: unknown): value is ResourceKey<Family> =>
    isResourceKeyParts(value) &&
    (!runtimeBrand ||
      (value as RuntimeResourceKey)[resourceKeyFamily] === token);
  const assert = (value: unknown): asserts value is ResourceKey<Family> => {
    if (!is(value)) throw new TypeError(`expected a ${name} resource key`);
  };
  const family: ResourceKeyFamily<Family> = {
    name,
    fromParts,
    fromJSON(value) {
      if (!isResourceKeyParts(value)) {
        throw new TypeError(`expected { lo, hi } for a ${name} resource key`);
      }
      return fromParts(value.lo, value.hi);
    },
    is,
    assert,
    equals(left, right) {
      return (
        left === right ||
        (!!left &&
          !!right &&
          is(left) &&
          is(right) &&
          left.lo === right.lo &&
          left.hi === right.hi)
      );
    },
    format(value) {
      return `${name}:${formatResourceKeyParts(value)}`;
    },
    table<Value>() {
      return new ResourceKeyTable<Family, Value>(family);
    },
  };
  return Object.freeze(family);
}
