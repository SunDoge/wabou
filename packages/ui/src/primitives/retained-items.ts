import { type Accessor, createEffect, createSignal, untrack } from "solid-js";

export interface RetainedItem<T, Key> {
  readonly key: Key;
  /** Latest source value for this key, including while it exits. */
  readonly value: Accessor<T>;
  /** False as soon as the key leaves the logical source. */
  readonly present: Accessor<boolean>;
}

export interface RetainedItems<T, Key> {
  /** Active entries plus entries waiting for their visual exit to finish. */
  readonly entries: Accessor<readonly RetainedItem<T, Key>[]>;
  /** Remove an absent key after its exit completes. */
  release(key: Key): boolean;
}

interface MutableRetainedItem<T, Key> extends RetainedItem<T, Key> {
  current: T;
}

const assertUniqueKeys = <T, Key>(
  items: readonly T[],
  key: (item: T) => Key,
): Map<Key, T> => {
  const keyed = new Map<Key, T>();
  for (const item of items) {
    const itemKey = key(item);
    if (keyed.has(itemKey)) {
      throw new Error(`duplicate retained item key: ${String(itemKey)}`);
    }
    keyed.set(itemKey, item);
  }
  return keyed;
};

/**
 * Keep keyed values mounted after logical removal until `release` is called.
 *
 * Entries are stable by key, expose the latest source value, and report
 * logical presence independently from visual retention. This is the common
 * lifecycle needed by exit animations without delaying state or semantics.
 */
export function createRetainedItems<T, Key>(
  source: Accessor<readonly T[]>,
  key: (item: T) => Key,
): RetainedItems<T, Key> {
  const [revision, setRevision] = createSignal(0, { ownedWrite: true });
  let active = assertUniqueKeys(untrack(source), key);

  const createEntry = (itemKey: Key, item: T): MutableRetainedItem<T, Key> => {
    const entry: MutableRetainedItem<T, Key> = {
      key: itemKey,
      current: item,
      value: () => {
        revision();
        return entry.current;
      },
      present: () => {
        revision();
        return active.has(itemKey);
      },
    };
    return entry;
  };

  const initial = [...active].map(([itemKey, item]) =>
    createEntry(itemKey, item),
  );
  const [entries, setEntries] = createSignal<
    readonly MutableRetainedItem<T, Key>[]
  >(initial, { ownedWrite: true });

  createEffect(source, (current) => {
    const nextActive = assertUniqueKeys(current, key);
    const previous = untrack(entries);
    const previousByKey = new Map(previous.map((entry) => [entry.key, entry]));
    const exiting = previous.filter((entry) => !nextActive.has(entry.key));
    const next = current.map((item) => {
      const itemKey = key(item);
      const entry = previousByKey.get(itemKey) ?? createEntry(itemKey, item);
      entry.current = item;
      return entry;
    });
    active = nextActive;
    setEntries([...exiting, ...next]);
    setRevision((value) => value + 1);
  });

  return {
    entries,
    release(itemKey) {
      if (active.has(itemKey)) return false;
      const previous = untrack(entries);
      const next = previous.filter((entry) => entry.key !== itemKey);
      if (next.length === previous.length) return false;
      setEntries(next);
      setRevision((value) => value + 1);
      return true;
    },
  };
}
