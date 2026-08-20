import { match } from "ts-pattern";
import {
  type Accessor,
  createEffect,
  createMemo,
  createSignal,
  untrack,
} from "solid-js";

export type SelectionMode = "single" | "multiple";
export type Selection = string | readonly string[] | undefined;

export function toggleSelection(
  current: Selection,
  item: string,
  mode: SelectionMode,
  allowEmpty = false,
): Selection {
  return match(mode)
    .with("single", () => (current === item && allowEmpty ? undefined : item))
    .with("multiple", () => {
      const values = Array.isArray(current) ? current : [];
      return values.includes(item)
        ? values.filter((value) => value !== item)
        : [...values, item];
    })
    .exhaustive();
}

export function isSelected(selection: Selection, item: string): boolean {
  return Array.isArray(selection)
    ? selection.includes(item)
    : selection === item;
}

export interface KeyedSelectionOptions<T, Key> {
  items: Accessor<readonly T[]>;
  key: (item: T) => Key;
  mode: SelectionMode;
  initialKeys?: Iterable<Key>;
}

export interface KeyedSelection<T, Key> {
  keys: Accessor<ReadonlySet<Key>>;
  items: Accessor<readonly T[]>;
  item: Accessor<T | undefined>;
  isSelected(key: Key): boolean;
  select(key: Key): void;
  deselect(key: Key): void;
  toggle(key: Key): void;
  set(keys: Iterable<Key>): void;
  clear(): void;
}

/**
 * Selection state owned by stable keys while values remain host-owned.
 * Selected items always resolve to the latest objects from `items`; keys that
 * disappear from the source are removed instead of becoming ghost selections.
 */
export function createKeyedSelection<T, Key>(
  options: KeyedSelectionOptions<T, Key>,
): KeyedSelection<T, Key> {
  const initialAvailable = new Set(untrack(options.items).map(options.key));
  const initial = normalizeKeys(
    options.initialKeys ?? [],
    initialAvailable,
    options.mode,
  );
  const [keys, setKeys] = createSignal<ReadonlySet<Key>>(initial);

  createEffect(
    () => new Set(options.items().map(options.key)),
    (available) => {
      setKeys((current) => {
        const next = normalizeKeys(current, available, options.mode);
        return setsEqual(current, next) ? current : next;
      });
    },
  );

  const selectedItems = createMemo(() => {
    const selected = keys();
    return options.items().filter((item) => selected.has(options.key(item)));
  });
  const set = (nextKeys: Iterable<Key>) => {
    const available = new Set(options.items().map(options.key));
    const next = normalizeKeys(nextKeys, available, options.mode);
    setKeys((current) => (setsEqual(current, next) ? current : next));
  };
  const select = (key: Key) => {
    if (options.mode === "single") {
      set([key]);
      return;
    }
    setKeys((current) => {
      if (current.has(key)) return current;
      const available = new Set(options.items().map(options.key));
      if (!available.has(key)) return current;
      return new Set([...current, key]);
    });
  };
  const deselect = (key: Key) => {
    setKeys((current) => {
      if (!current.has(key)) return current;
      const next = new Set(current);
      next.delete(key);
      return next;
    });
  };

  return {
    keys,
    items: selectedItems,
    item: () => selectedItems()[0],
    isSelected: (key) => keys().has(key),
    select,
    deselect,
    toggle: (key) => (keys().has(key) ? deselect(key) : select(key)),
    set,
    clear: () =>
      setKeys((current) => (current.size === 0 ? current : new Set())),
  };
}

function normalizeKeys<Key>(
  keys: Iterable<Key>,
  available: ReadonlySet<Key>,
  mode: SelectionMode,
): ReadonlySet<Key> {
  const next = new Set<Key>();
  for (const key of keys) {
    if (!available.has(key)) continue;
    next.add(key);
    if (mode === "single") break;
  }
  return next;
}

function setsEqual<Key>(
  left: ReadonlySet<Key>,
  right: ReadonlySet<Key>,
): boolean {
  if (left.size !== right.size) return false;
  for (const key of left) if (!right.has(key)) return false;
  return true;
}
