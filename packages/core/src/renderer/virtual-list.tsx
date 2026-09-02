import {
  type Accessor,
  createMemo,
  For as ForValue,
  type JSX,
  untrack,
} from "solid-js";
import { mergeClasses } from "../style";
import type { WabouSemanticRole } from "./index";

export interface VirtualListProps<T> {
  /** Accessor for the full backing array. Only the visible slice renders. */
  items: () => readonly T[];
  /** Fixed height of every row, in logical pixels. */
  itemHeight: number;
  /**
   * Visible viewport height in logical pixels. When omitted, the list fills
   * its bounded parent and observes its completed native layout size.
   */
  viewportHeight?: number;
  /** Classes applied to the native scroll viewport. */
  class?: string;
  /** Stable application identity. Required so refreshed objects do not remount rows. */
  getItemKey: (item: T, index: number) => string | number;
  /** Explicit semantic role for the viewport, such as `listbox`. */
  role?: WabouSemanticRole;
  /** Accessible name for the native scroll viewport. */
  accessibilityLabel?: string;
  /** Render a single row given its item and absolute index. */
  children: (item: Accessor<T>, index: Accessor<number>) => JSX.Element;
}

export function createVirtualRow<T>(
  items: () => readonly T[],
  index: () => number,
) {
  return createMemo(() => items()[index()]);
}

const encodedItemKey = (key: string | number) =>
  typeof key === "number" ? `number:${key}` : `string:${key}`;

export function validateVirtualItemKeys<T>(
  items: readonly T[],
  getItemKey: (item: T, index: number) => string | number,
) {
  const keys = new Array<string | number>(items.length);
  const seen = new Set<string>();
  for (let index = 0; index < items.length; index++) {
    const item = items[index];
    if (item === undefined)
      throw new TypeError(`VirtualList item at index ${index} is undefined`);
    const key = getItemKey(item, index);
    if (typeof key === "number" && !Number.isFinite(key))
      throw new TypeError(`VirtualList key at index ${index} must be finite`);
    const encoded = encodedItemKey(key);
    if (seen.has(encoded))
      throw new TypeError(
        `VirtualList key ${JSON.stringify(key)} is duplicated at index ${index}`,
      );
    seen.add(encoded);
    keys[index] = key;
  }
  return keys;
}

export function createVirtualItemIdentity<T>(
  items: () => readonly T[],
  index: () => number,
  getItemKey: (item: T, index: number) => string | number,
) {
  return createMemo<{ key: string } | undefined>(
    () => {
      const currentIndex = index();
      const item = items()[currentIndex];
      return item === undefined
        ? undefined
        : { key: encodedItemKey(getItemKey(item, currentIndex)) };
    },
    {
      equals: (
        previous: { key: string } | undefined,
        next: { key: string } | undefined,
      ) => previous?.key === next?.key,
    },
  );
}

/**
 * Uniform list whose viewport, scroll state, visible range, layout and paint
 * are owned by GPUI. Solid retains stable row subtrees so reactive updates keep
 * their ordinary component semantics; GPUI materializes only visible rows.
 */
export function VirtualList<T>(props: VirtualListProps<T>): JSX.Element {
  const config = untrack(() => ({
    items: props.items,
    children: props.children,
    itemHeight: props.itemHeight,
    viewportHeight: props.viewportHeight,
    class: props.class,
    getItemKey: props.getItemKey,
    role: props.role,
    accessibilityLabel: props.accessibilityLabel,
  }));
  const itemKeys = createMemo(() => {
    return validateVirtualItemKeys(config.items(), config.getItemKey);
  });

  return (
    <virtual-list
      projectionBoundary
      class={mergeClasses(
        "w-full min-w-0 min-h-0 overflow-x-hidden overflow-y-auto",
        config.class,
      )}
      role={config.role}
      aria-label={config.accessibilityLabel}
      style={{
        ...(config.viewportHeight === undefined
          ? {}
          : { height: `${config.viewportHeight}px` }),
        width: "100%",
      }}
    >
      <ForValue each={config.items()} keyed={false}>
        {(_value, index) => {
          const rowIndex = () => index;
          const item = createVirtualRow(config.items, rowIndex);
          const identity = createVirtualItemIdentity(
            config.items,
            rowIndex,
            (_item, currentIndex) => itemKeys()[currentIndex] ?? currentIndex,
          );
          return (
            <view
              style={{
                height: `${config.itemHeight}px`,
                "flex-shrink": 0,
                width: "100%",
              }}
            >
              {identity() &&
                config.children(() => {
                  const current = item();
                  if (current === undefined)
                    throw new Error(
                      "VirtualList item disappeared while its row was mounted",
                    );
                  return current;
                }, rowIndex)}
            </view>
          );
        }}
      </ForValue>
    </virtual-list>
  );
}
