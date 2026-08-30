import { type Accessor, createSignal, onCleanup } from "solid-js";
import { type Host, useHost } from "../renderer/host";
import { bindCapability, type NativeCapability } from "./native-capability";

/** JSON-compatible values stored by the built-in KV service. */
export type KvValue =
  | null
  | boolean
  | number
  | string
  | readonly KvValue[]
  | { readonly [key: string]: KvValue };

/** One component of a hierarchical key. */
export type KvKeyPart = string | number | boolean | Uint8Array;
/** Hierarchical key whose array boundaries define namespaces. */
export type KvKey = readonly KvKeyPart[];
/** Decimal revision assigned by SQLite to an atomic commit. */
export type KvVersionstamp = string;

export interface KvEntry<T extends KvValue = KvValue> {
  readonly key: KvKey;
  readonly value: T;
  readonly versionstamp: KvVersionstamp;
  readonly expiresAt?: number;
}

export interface KvSetOptions {
  /** Remove the entry after this many milliseconds. */
  expireIn?: number;
}

export interface KvListOptions {
  prefix?: KvKey;
  limit?: number;
  reverse?: boolean;
}

export interface KvCheck {
  key: KvKey;
  /** `null` requires the key to be absent. */
  versionstamp: KvVersionstamp | null;
}

export interface KvCommitResult {
  readonly committed: boolean;
  readonly versionstamp?: KvVersionstamp;
}

type WireKeyPart =
  | { type: "string"; value: string }
  | { type: "i64"; value: string }
  | { type: "bytes"; value: number[] }
  | { type: "bool"; value: boolean };

interface WireEntry {
  key: WireKeyPart[];
  value: KvValue;
  versionstamp: string;
  expiresAt: number | null;
}

interface NativeKvCapability extends NativeCapability {
  get(request: { key: WireKeyPart[] }): Promise<WireEntry | null>;
  set(request: {
    key: WireKeyPart[];
    value: KvValue;
    expireIn?: number;
  }): Promise<{ versionstamp: string }>;
  delete(request: { key: WireKeyPart[] }): Promise<{ versionstamp: string }>;
  list(request: {
    prefix: WireKeyPart[];
    limit: number;
    reverse: boolean;
  }): Promise<WireEntry[]>;
  atomic(request: {
    checks: { key: WireKeyPart[]; versionstamp: string | null }[];
    mutations: (
      | {
          type: "set";
          key: WireKeyPart[];
          value: KvValue;
          expireIn?: number;
        }
      | { type: "delete"; key: WireKeyPart[] }
    )[];
  }): Promise<{ committed: boolean; versionstamp: string | null }>;
}

interface KvHost extends Host {
  kv: NativeKvCapability;
}

/** Fluent optimistic transaction committed as one SQLite transaction. */
export class KvAtomicOperation {
  readonly #prefix: KvKey;
  readonly #native: NativeKvCapability;
  readonly #checks: { key: WireKeyPart[]; versionstamp: string | null }[] = [];
  readonly #mutations: (
    | {
        type: "set";
        key: WireKeyPart[];
        value: KvValue;
        expireIn?: number;
      }
    | { type: "delete"; key: WireKeyPart[] }
  )[] = [];

  constructor(prefix: KvKey, native: NativeKvCapability) {
    this.#prefix = prefix;
    this.#native = native;
  }

  check(check: KvCheck | Pick<KvEntry, "key" | "versionstamp">): this {
    this.#checks.push({
      key: encodeKey(scopedKey(this.#prefix, check.key)),
      versionstamp: check.versionstamp,
    });
    return this;
  }

  set(key: KvKey, value: KvValue, options: KvSetOptions = {}): this {
    this.#mutations.push({
      type: "set",
      key: encodeKey(scopedKey(this.#prefix, key)),
      value,
      ...encodeExpiry(options),
    });
    return this;
  }

  delete(key: KvKey): this {
    this.#mutations.push({
      type: "delete",
      key: encodeKey(scopedKey(this.#prefix, key)),
    });
    return this;
  }

  async commit(): Promise<KvCommitResult> {
    const result = await this.#native.atomic({
      checks: this.#checks,
      mutations: this.#mutations,
    });
    return {
      committed: result.committed,
      ...(result.versionstamp === null
        ? {}
        : { versionstamp: result.versionstamp }),
    };
  }
}

/** Application-scoped view of the built-in SQLite KV service. */
export interface Kv {
  get<T extends KvValue = KvValue>(key: KvKey): Promise<KvEntry<T> | null>;
  set(
    key: KvKey,
    value: KvValue,
    options?: KvSetOptions,
  ): Promise<KvVersionstamp>;
  delete(key: KvKey): Promise<KvVersionstamp>;
  list<T extends KvValue = KvValue>(
    options?: KvListOptions,
  ): AsyncIterable<KvEntry<T>>;
  atomic(): KvAtomicOperation;
}

export interface KvSignal<T extends KvValue> {
  /** Current local value; available immediately. */
  readonly value: Accessor<T>;
  /** Whether the initial durable read has settled. */
  readonly ready: Accessor<boolean>;
  /** Most recent load or write failure. */
  readonly error: Accessor<unknown>;
  /** Update locally and schedule persistence. */
  set(next: T | ((previous: T) => T)): void;
  /** Reload without overwriting a newer local edit. */
  reload(): Promise<void>;
  /** Immediately persist the latest pending value. */
  flush(): Promise<void>;
}

/**
 * Bind one explicit KV key to Solid state.
 *
 * The key is deliberately required: source location, signal creation order,
 * and variable names are not stable persistence identities across HMR or
 * refactors.
 */
export function createKvSignal<T extends KvValue>(options: {
  kv: Kv;
  key: KvKey;
  initial: T;
  saveDelayMs?: number;
}): KvSignal<T> {
  // `T` is JSON-compatible and therefore cannot be a function. Passing a
  // function here opts into Solid 2's lazy/derived signal overload, whose
  // value is intentionally not replaced by later writes. The cast selects
  // Solid's non-function overload; the `KvValue` bound proves it is safe.
  const [value, setValue] = createSignal<T>(options.initial as never, {
    ownedWrite: true,
  });
  const [ready, setReady] = createSignal(false);
  const [error, setError] = createSignal<unknown>();
  let generation = 0;
  let pending: T | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let writer: Promise<void> | undefined;

  const reload = async () => {
    const startedAt = generation;
    try {
      const entry = await options.kv.get<T>(options.key);
      if (generation === startedAt && entry !== null)
        setValue(() => entry.value);
      setError(undefined);
    } catch (cause) {
      setError(cause);
    } finally {
      setReady(true);
    }
  };

  const drain = async () => {
    while (pending !== undefined) {
      const next = pending;
      pending = undefined;
      try {
        await options.kv.set(options.key, next);
        setError(undefined);
      } catch (cause) {
        if (pending === undefined) pending = next;
        setError(cause);
        throw cause;
      }
    }
  };

  const flush = (): Promise<void> => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
    if (writer) return writer;
    writer = drain().finally(() => {
      writer = undefined;
    });
    return writer;
  };

  const set = (next: T | ((previous: T) => T)) => {
    const resolved =
      typeof next === "function" ? (next as (previous: T) => T)(value()) : next;
    generation += 1;
    setValue(() => resolved);
    pending = resolved;
    if (timer !== undefined) clearTimeout(timer);
    timer = setTimeout(
      () => void flush().catch(() => {}),
      options.saveDelayMs ?? 150,
    );
  };

  void reload();
  onCleanup(() => {
    void flush().catch(() => {});
  });
  return { value, ready, error, set, reload, flush };
}

/**
 * Open a namespaced view of the host's SQLite store.
 *
 * The host must opt in with `HostBuilder::kv()` and configure stable app
 * directories. Prefixes are prepended by whole key parts, never string joined.
 */
export function openKv(prefix: KvKey = []): Kv {
  const native = bindCapability(useHost<KvHost>().kv, {
    name: "kv",
    version: 1,
  });
  const namespace = [...prefix];
  for (const part of namespace) encodePart(part);

  return {
    async get<T extends KvValue>(key: KvKey) {
      const entry = await native.get({
        key: encodeKey(scopedKey(namespace, key)),
      });
      return entry === null ? null : decodeEntry<T>(entry, namespace.length);
    },
    async set(key, value, options = {}) {
      const result = await native.set({
        key: encodeKey(scopedKey(namespace, key)),
        value,
        ...encodeExpiry(options),
      });
      return result.versionstamp;
    },
    async delete(key) {
      const result = await native.delete({
        key: encodeKey(scopedKey(namespace, key)),
      });
      return result.versionstamp;
    },
    async *list<T extends KvValue>(options: KvListOptions = {}) {
      const limit = options.limit ?? 100;
      if (!Number.isSafeInteger(limit) || limit < 0)
        throw new RangeError(
          "KV list limit must be a non-negative safe integer",
        );
      const entries = await native.list({
        prefix: encodeKey([...namespace, ...(options.prefix ?? [])], true),
        limit,
        reverse: options.reverse ?? false,
      });
      for (const entry of entries)
        yield decodeEntry<T>(entry, namespace.length);
    },
    atomic: () => new KvAtomicOperation(namespace, native),
  };
}

function scopedKey(prefix: KvKey, key: KvKey): KvKey {
  const scoped = [...prefix, ...key];
  if (scoped.length === 0)
    throw new TypeError("KV keys must contain at least one part");
  return scoped;
}

function encodeKey(key: KvKey, allowEmpty = false): WireKeyPart[] {
  if (!allowEmpty && key.length === 0)
    throw new TypeError("KV keys must contain at least one part");
  return key.map(encodePart);
}

function encodePart(part: KvKeyPart): WireKeyPart {
  if (typeof part === "string") return { type: "string", value: part };
  if (typeof part === "boolean") return { type: "bool", value: part };
  if (typeof part === "number") {
    if (!Number.isSafeInteger(part))
      throw new RangeError("numeric KV key parts must be safe integers");
    return { type: "i64", value: String(part) };
  }
  if (part instanceof Uint8Array)
    return { type: "bytes", value: Array.from(part) };
  throw new TypeError("unsupported KV key part");
}

function decodePart(part: WireKeyPart): KvKeyPart {
  switch (part.type) {
    case "string":
    case "bool":
      return part.value;
    case "i64": {
      const value = Number(part.value);
      if (!Number.isSafeInteger(value))
        throw new RangeError(
          `KV integer ${part.value} is not safe in JavaScript`,
        );
      return value;
    }
    case "bytes":
      return Uint8Array.from(part.value);
  }
}

function decodeEntry<T extends KvValue>(
  entry: WireEntry,
  prefixLength: number,
): KvEntry<T> {
  return {
    key: entry.key.slice(prefixLength).map(decodePart),
    value: entry.value as T,
    versionstamp: entry.versionstamp,
    ...(entry.expiresAt === null ? {} : { expiresAt: entry.expiresAt }),
  };
}

function encodeExpiry(options: KvSetOptions): { expireIn?: number } {
  if (options.expireIn === undefined) return {};
  if (!Number.isSafeInteger(options.expireIn) || options.expireIn < 0)
    throw new RangeError("KV expireIn must be a non-negative safe integer");
  return { expireIn: options.expireIn };
}
