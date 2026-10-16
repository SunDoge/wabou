import { Accessor } from "solid-js";
//#region src/primitives/retained-items.d.ts
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
/**
 * Keep keyed values mounted after logical removal until `release` is called.
 *
 * Entries are stable by key, expose the latest source value, and report
 * logical presence independently from visual retention. This is the common
 * lifecycle needed by exit animations without delaying state or semantics.
 */
export declare function createRetainedItems<T, Key>(source: Accessor<readonly T[]>, key: (item: T) => Key): RetainedItems<T, Key>;
//#endregion
//# sourceMappingURL=retained-items.d.mts.map