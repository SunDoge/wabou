import { Accessor } from "solid-js";
//#region src/primitives/interactions/selection.d.ts
export type SelectionMode = "single" | "multiple";
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
export declare function createKeyedSelection<T, Key>(options: KeyedSelectionOptions<T, Key>): KeyedSelection<T, Key>;
//#endregion
//# sourceMappingURL=selection.d.mts.map