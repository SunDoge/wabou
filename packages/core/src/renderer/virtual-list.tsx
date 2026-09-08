import {
  type Accessor,
  createEffect,
  createMemo,
  createSignal,
  For as ForValue,
  type JSX,
  onCleanup,
  untrack,
} from "solid-js";
import { mergeClasses } from "../style";
import type { Handle, WabouScrollEvent, WabouSemanticRole } from "./index";

export interface VirtualListProps<T> {
  /** Accessor for the full backing array. Only the visible slice renders. */
  items: () => readonly T[];
  /** Fixed height of every row, in logical pixels. */
  itemHeight: number;
  /** Extra rows retained before and after the visible range. Defaults to 2. */
  overscan?: number;
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
  /** Receives the stable imperative controller for keyboard/focus integration. */
  controllerRef?(controller: VirtualListController): void;
  /** Called when the mounted, overscanned range changes. `end` is exclusive. */
  onVisibleRangeChange?(range: VirtualListRange): void;
  /** Render a single row given its item and absolute index. */
  children: (item: Accessor<T>, index: Accessor<number>) => JSX.Element;
}

export type VirtualListScrollAlignment = "nearest" | "start" | "center" | "end";

export interface VirtualListController {
  scrollToIndex(index: number, alignment?: VirtualListScrollAlignment): void;
}

export function createVirtualRow<T>(
  items: () => readonly T[],
  index: () => number,
) {
  return createMemo(() => items()[index()]);
}

const encodedItemKey = (key: string | number) =>
  typeof key === "number" ? `number:${key}` : `string:${key}`;

export interface VirtualListRange {
  readonly start: number;
  readonly end: number;
}

interface VirtualRow<T> {
  readonly index: number;
  readonly key: string;
  readonly item: T;
}

export function calculateVirtualRange(
  itemCount: number,
  itemHeight: number,
  viewportHeight: number,
  scrollTop: number,
  overscan: number,
): VirtualListRange {
  if (!Number.isSafeInteger(itemCount) || itemCount < 0)
    throw new RangeError(
      "VirtualList item count must be a non-negative integer",
    );
  if (!Number.isFinite(itemHeight) || itemHeight <= 0)
    throw new RangeError("VirtualList itemHeight must be positive and finite");
  if (!Number.isFinite(viewportHeight) || viewportHeight < 0)
    throw new RangeError(
      "VirtualList viewport height must be finite and non-negative",
    );
  if (!Number.isFinite(scrollTop))
    throw new RangeError("VirtualList scroll offset must be finite");
  if (!Number.isSafeInteger(overscan) || overscan < 0)
    throw new RangeError("VirtualList overscan must be a non-negative integer");

  const visibleCount = Math.max(1, Math.ceil(viewportHeight / itemHeight));
  const maxFirstVisible = Math.max(0, itemCount - visibleCount);
  const firstVisible = Math.min(
    maxFirstVisible,
    Math.floor(Math.max(0, scrollTop) / itemHeight),
  );
  const start = Math.max(0, Math.min(itemCount, firstVisible - overscan));
  const end = Math.max(
    start,
    Math.min(itemCount, firstVisible + visibleCount + overscan),
  );
  return { start, end };
}

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
 * Uniform-height list that keeps scrolling and clipping native while Solid
 * mounts only the visible rows plus a small overscan. Spacer nodes preserve the
 * complete native scroll extent without creating offscreen item subtrees.
 */
export function VirtualList<T>(props: VirtualListProps<T>): JSX.Element {
  const config = untrack(() => ({
    items: props.items,
    children: props.children,
    itemHeight: props.itemHeight,
    overscan: props.overscan ?? 2,
    viewportHeight: props.viewportHeight,
    class: props.class,
    getItemKey: props.getItemKey,
    role: props.role,
    accessibilityLabel: props.accessibilityLabel,
    controllerRef: props.controllerRef,
    onVisibleRangeChange: props.onVisibleRangeChange,
  }));
  const source = createMemo(() => {
    const items = config.items();
    return {
      items,
      keys: validateVirtualItemKeys(items, config.getItemKey),
    };
  });
  const [scrollTop, setScrollTop] = createSignal(0);
  const [measuredHeight, setMeasuredHeight] = createSignal(0);
  let viewport: Handle | undefined;
  let observer: ResizeObserver | undefined;

  const viewportHeight = () => config.viewportHeight ?? measuredHeight();
  const range = createMemo(
    () =>
      calculateVirtualRange(
        source().items.length,
        config.itemHeight,
        viewportHeight(),
        scrollTop(),
        config.overscan,
      ),
    {
      equals: (previous, next) =>
        previous.start === next.start && previous.end === next.end,
    },
  );
  const visibleRows = createMemo<readonly VirtualRow<T>[]>(() => {
    const currentRange = range();
    const { items, keys } = source();
    const rows = new Array<VirtualRow<T>>(
      currentRange.end - currentRange.start,
    );
    for (let index = currentRange.start; index < currentRange.end; index++) {
      const item = items[index];
      if (item === undefined)
        throw new Error("VirtualList item snapshot changed during projection");
      rows[index - currentRange.start] = {
        index,
        key: encodedItemKey(keys[index] ?? index),
        item,
      };
    }
    return rows;
  });
  const observeViewport = (node: Handle) => {
    viewport = node;
    observer?.disconnect();
    if (config.viewportHeight !== undefined) return;
    observer = new ResizeObserver(([entry]) => {
      if (entry) setMeasuredHeight(entry.contentRect.height);
    });
    observer.observe(node as never);
  };
  const handleScroll = (event: WabouScrollEvent) => {
    if (event.scrollY !== undefined) setScrollTop(Math.max(0, event.scrollY));
  };

  const controller: VirtualListController = {
    scrollToIndex(index, alignment = "nearest") {
      const itemCount = source().items.length;
      if (!Number.isSafeInteger(index) || index < 0 || index >= itemCount) {
        throw new RangeError(
          `VirtualList index ${index} is outside 0..${Math.max(0, itemCount - 1)}`,
        );
      }
      const height = viewportHeight();
      const itemTop = index * config.itemHeight;
      const itemBottom = itemTop + config.itemHeight;
      const currentTop = scrollTop();
      const currentBottom = currentTop + height;
      const requested =
        alignment === "start"
          ? itemTop
          : alignment === "center"
            ? itemTop - (height - config.itemHeight) / 2
            : alignment === "end"
              ? itemBottom - height
              : itemTop < currentTop
                ? itemTop
                : itemBottom > currentBottom
                  ? itemBottom - height
                  : currentTop;
      const next = Math.min(
        Math.max(0, itemCount * config.itemHeight - height),
        Math.max(0, requested),
      );
      setScrollTop(next);
      viewport?.scrollTo({ top: next });
    },
  };
  config.controllerRef?.(controller);
  createEffect(range, (next) => {
    untrack(() => config.onVisibleRangeChange?.(next));
  });

  onCleanup(() => observer?.disconnect());

  return (
    <virtual-list
      projectionBoundary
      class={mergeClasses(
        "w-full min-w-0 min-h-0 overflow-x-hidden overflow-y-auto",
        config.class,
      )}
      role={config.role}
      aria-label={config.accessibilityLabel}
      ref={observeViewport}
      onScroll={handleScroll}
      style={{
        ...(config.viewportHeight === undefined
          ? {}
          : { height: `${config.viewportHeight}px` }),
        width: "100%",
      }}
    >
      <view
        aria-hidden
        style={{
          height: `${range().start * config.itemHeight}px`,
          "flex-shrink": 0,
          width: "100%",
        }}
      />
      <ForValue each={visibleRows()} keyed={(row) => row.key}>
        {(row) => {
          const rowIndex = () => row().index;
          return (
            <view
              style={{
                height: `${config.itemHeight}px`,
                "flex-shrink": 0,
                width: "100%",
              }}
            >
              {config.children(() => row().item, rowIndex)}
            </view>
          );
        }}
      </ForValue>
      <view
        aria-hidden
        style={{
          height: `${(source().items.length - range().end) * config.itemHeight}px`,
          "flex-shrink": 0,
          width: "100%",
        }}
      />
    </virtual-list>
  );
}
