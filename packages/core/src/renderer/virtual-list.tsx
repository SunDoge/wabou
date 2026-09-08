import {
  Virtualizer,
  type VirtualizerOptions,
  type Rect as VirtualRect,
} from "@tanstack/virtual-core";
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
import type {
  Handle,
  WabouKeyEvent,
  WabouScrollEvent,
  WabouSemanticRole,
} from "./index";

export interface VirtualListProps<T> {
  /** Accessor for the full backing array. Only the visible slice renders. */
  items: () => readonly T[];
  /** Fixed height of every row, in logical pixels. */
  itemHeight: number;
  /** Extra rows retained before and after the visible range. Defaults to 2. */
  overscan?: number;
  /** Stable edge when items are inserted or removed. Defaults to `start`. */
  anchorTo?: "start" | "end";
  /** Follow appended items while an end-anchored viewport is already at its end. */
  followOnAppend?: boolean | "auto" | "smooth" | "instant";
  /** Logical pixels from the end that still count as end-pinned. */
  scrollEndThreshold?: number;
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
  /** Native tab order for list-owned keyboard navigation. */
  focusOrder?: number;
  /** Semantic descendant currently owned by a focused composite list. */
  "aria-activedescendant"?: string;
  /** Keyboard handler for listbox, tree, command, or application navigation. */
  onKeyDown?(event: WabouKeyEvent): void;
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
  scrollToEnd(behavior?: "auto" | "smooth" | "instant"): void;
  isAtEnd(threshold?: number): boolean;
  getDistanceFromEnd(): number;
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
  readonly start: number;
  readonly end: number;
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
    anchorTo: props.anchorTo ?? "start",
    followOnAppend: props.followOnAppend ?? false,
    scrollEndThreshold: props.scrollEndThreshold ?? 1,
    viewportHeight: props.viewportHeight,
    class: props.class,
    getItemKey: props.getItemKey,
    role: props.role,
    accessibilityLabel: props.accessibilityLabel,
    focusOrder: props.focusOrder,
    // Composite focus follows application selection, so keep this semantic
    // reference reactive even though the structural list configuration is
    // intentionally captured once.
    activeDescendant: () => props["aria-activedescendant"],
    onKeyDown: props.onKeyDown,
    controllerRef: props.controllerRef,
    onVisibleRangeChange: props.onVisibleRangeChange,
  }));
  if (!Number.isFinite(config.itemHeight) || config.itemHeight <= 0) {
    throw new RangeError("VirtualList itemHeight must be positive and finite");
  }
  if (!Number.isSafeInteger(config.overscan) || config.overscan < 0) {
    throw new RangeError("VirtualList overscan must be a non-negative integer");
  }
  if (
    config.viewportHeight !== undefined &&
    (!Number.isFinite(config.viewportHeight) || config.viewportHeight < 0)
  ) {
    throw new RangeError(
      "VirtualList viewportHeight must be finite and non-negative",
    );
  }
  if (
    !Number.isFinite(config.scrollEndThreshold) ||
    config.scrollEndThreshold < 0
  ) {
    throw new RangeError(
      "VirtualList scrollEndThreshold must be finite and non-negative",
    );
  }
  const source = createMemo(() => {
    const items = config.items();
    return {
      items,
      keys: validateVirtualItemKeys(items, config.getItemKey),
    };
  });
  const [measuredHeight, setMeasuredHeight] = createSignal(0, {
    ownedWrite: true,
  });
  const [virtualRevision, setVirtualRevision] = createSignal(0, {
    ownedWrite: true,
  });
  let viewport: Handle | undefined;
  let observer: ResizeObserver | undefined;
  let reportRect: ((rect: VirtualRect) => void) | undefined;
  let reportOffset:
    | ((offset: number, isScrolling: boolean) => void)
    | undefined;

  const viewportHeight = () => config.viewportHeight ?? measuredHeight();
  let sourceSnapshot = untrack(source);
  let currentViewportHeight = Math.max(
    config.itemHeight,
    untrack(viewportHeight),
  );
  let currentScrollOffset = 0;
  const scrollTarget = {
    ownerDocument: {
      defaultView: globalThis as unknown as Window & typeof globalThis,
    },
    scrollHeight: sourceSnapshot.items.length * config.itemHeight,
    scrollWidth: 0,
    clientHeight: currentViewportHeight,
    clientWidth: 0,
  };
  const virtualizerOptions = (): VirtualizerOptions<Element, Element> => {
    const snapshot = sourceSnapshot;
    const viewportHeight = currentViewportHeight;
    const initialOffset = currentScrollOffset;
    return {
      count: snapshot.items.length,
      getScrollElement: () => scrollTarget as unknown as Element,
      estimateSize: () => config.itemHeight,
      getItemKey: (index) => snapshot.keys[index] ?? index,
      overscan: config.overscan,
      anchorTo: config.anchorTo,
      followOnAppend: config.followOnAppend,
      scrollEndThreshold: config.scrollEndThreshold,
      initialRect: { width: 0, height: viewportHeight },
      initialOffset,
      scrollToFn: (offset) => {
        const next = Math.max(0, offset);
        currentScrollOffset = next;
        reportOffset?.(next, false);
        viewport?.scrollTo({ top: next });
      },
      observeElementRect: (_instance, callback) => {
        reportRect = callback;
        callback({ width: 0, height: currentViewportHeight });
        return () => {
          if (reportRect === callback) reportRect = undefined;
        };
      },
      observeElementOffset: (_instance, callback) => {
        reportOffset = callback;
        callback(currentScrollOffset, false);
        return () => {
          if (reportOffset === callback) reportOffset = undefined;
        };
      },
      onChange: () => setVirtualRevision((revision) => revision + 1),
    };
  };
  const virtualizer = new Virtualizer<Element, Element>(virtualizerOptions());
  virtualizer._willUpdate();
  const disposeVirtualizer = virtualizer._didMount();
  const virtualInputs = createMemo(
    () => ({ source: source(), height: viewportHeight() }),
    {
      equals: (previous, next) =>
        previous.source === next.source && previous.height === next.height,
    },
  );
  createEffect(virtualInputs, ({ source: nextSource, height }) => {
    sourceSnapshot = nextSource;
    currentViewportHeight = Math.max(config.itemHeight, height);
    virtualizer.setOptions(virtualizerOptions());
    scrollTarget.scrollHeight = sourceSnapshot.items.length * config.itemHeight;
    scrollTarget.clientHeight = currentViewportHeight;
    reportRect?.({ width: 0, height: currentViewportHeight });
    const maxOffset = Math.max(
      0,
      scrollTarget.scrollHeight - currentViewportHeight,
    );
    if (currentScrollOffset > maxOffset) {
      currentScrollOffset = maxOffset;
      reportOffset?.(maxOffset, false);
      viewport?.scrollTo({ top: maxOffset });
    }
    virtualizer._willUpdate();
  });
  const virtualWindow = createMemo(() => {
    virtualRevision();
    return {
      items: virtualizer.getVirtualItems(),
      totalSize: virtualizer.getTotalSize(),
    };
  });
  const range = createMemo<VirtualListRange>(
    () => {
      const items = virtualWindow().items;
      const first = items[0];
      const last = items.at(-1);
      return {
        start: first?.index ?? 0,
        end: last === undefined ? 0 : last.index + 1,
      };
    },
    {
      equals: (previous, next) =>
        previous.start === next.start && previous.end === next.end,
    },
  );
  const visibleRows = createMemo<readonly VirtualRow<T>[]>(() => {
    const window = virtualWindow();
    // A source replacement can keep the same virtual index window (for
    // example filtering rows 0..2 down to one result). TanStack then has no
    // reason to emit `onChange`, so depend on the Solid source as well as the
    // imperative virtual window to replace row contents in this flush. During
    // that source transition the old window can briefly contain indexes that
    // no longer exist; omit those until TanStack publishes its next window.
    const { items, keys } = source();
    const rows: VirtualRow<T>[] = [];
    for (const virtualItem of window.items) {
      const index = virtualItem.index;
      const item = items[index];
      if (item === undefined) continue;
      rows.push({
        index,
        key: encodedItemKey(keys[index] ?? index),
        item,
        start: virtualItem.start,
        end: virtualItem.end,
      });
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
    if (event.scrollY === undefined) return;
    const next = Math.max(0, event.scrollY);
    currentScrollOffset = next;
    reportOffset?.(next, true);
  };

  const controller: VirtualListController = {
    scrollToIndex(index, alignment = "nearest") {
      const itemCount = sourceSnapshot.items.length;
      if (!Number.isSafeInteger(index) || index < 0 || index >= itemCount) {
        throw new RangeError(
          `VirtualList index ${index} is outside 0..${Math.max(0, itemCount - 1)}`,
        );
      }
      virtualizer.scrollToIndex(index, {
        align: alignment === "nearest" ? "auto" : alignment,
      });
      // Programmatic navigation can run inside the same native key event that
      // owns the current Solid flush. Materialize TanStack's new window now;
      // waiting for a later native scroll echo leaves the active descendant
      // selected while its row is still unmounted for one frame.
      virtualizer._willUpdate();
      setVirtualRevision((revision) => revision + 1);
    },
    scrollToEnd(behavior = "auto") {
      virtualizer.scrollToEnd({ behavior });
      virtualizer._willUpdate();
      setVirtualRevision((revision) => revision + 1);
    },
    isAtEnd: (threshold) => virtualizer.isAtEnd(threshold),
    getDistanceFromEnd: () => virtualizer.getDistanceFromEnd(),
  };
  config.controllerRef?.(controller);
  createEffect(range, (next) => {
    untrack(() => config.onVisibleRangeChange?.(next));
  });

  onCleanup(() => {
    observer?.disconnect();
    disposeVirtualizer();
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
      aria-activedescendant={config.activeDescendant()}
      focusOrder={config.focusOrder}
      ref={observeViewport}
      onScroll={handleScroll}
      onKeyDown={config.onKeyDown}
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
          height: `${visibleRows()[0]?.start ?? 0}px`,
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
          height: `${Math.max(
            0,
            virtualWindow().totalSize - (visibleRows().at(-1)?.end ?? 0),
          )}px`,
          "flex-shrink": 0,
          width: "100%",
        }}
      />
    </virtual-list>
  );
}
