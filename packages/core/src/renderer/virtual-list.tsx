import { Virtualizer, type VirtualizerOptions } from "@tanstack/virtual-core";
import {
  type Accessor,
  createMemo,
  createSignal,
  For,
  type JSX,
  onCleanup,
  Show,
} from "solid-js";
import type { Handle, WabouSemanticRole } from "./index";

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
  /** Extra rows rendered above/below the viewport. Defaults to 4. */
  overscan?: number;
  /** Stable application identity. Required so refreshed objects do not remount rows. */
  getItemKey: (item: T, index: number) => string | number;
  /** Explicit semantic role for the viewport, such as `listbox`. */
  role?: WabouSemanticRole;
  /** Accessible name for the native scroll viewport. */
  accessibilityLabel?: string;
  /** Render a single row given its item and absolute index. */
  children: (item: Accessor<T>, index: Accessor<number>) => JSX.Element;
}

interface ScrollEvent {
  scrollX?: number;
  scrollY?: number;
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
 * Windowed Solid list backed by TanStack Virtual's framework-neutral core.
 * Rust remains authoritative for scrolling, clipping, hit testing and the
 * native scrollbar; this adapter supplies viewport/offset observations instead
 * of relying on HTMLElement, ResizeObserver or getBoundingClientRect().
 */
export function VirtualList<T>(props: VirtualListProps<T>): JSX.Element {
  const surface = {} as Element;
  let scrollHandle: Handle | undefined;
  let publishOffset: ((offset: number, scrolling: boolean) => void) | undefined;
  let publishRect:
    | ((rect: { width: number; height: number }) => void)
    | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let scrollEndTimer: ReturnType<typeof setTimeout> | undefined;
  let lastOffset = 0;
  const [version, invalidate] = createSignal(0, { equals: false });
  const [measuredRect, setMeasuredRect] = createSignal({
    width: 0,
    height: 0,
  });
  const viewportHeight = () => props.viewportHeight ?? measuredRect().height;
  const itemKeys = createMemo(() =>
    validateVirtualItemKeys(props.items(), props.getItemKey),
  );

  const options = (): VirtualizerOptions<Element, Element> => ({
    count: itemKeys().length,
    getItemKey: (index) => itemKeys()[index] ?? index,
    getScrollElement: () => (scrollHandle ? surface : null),
    estimateSize: () => props.itemHeight,
    overscan: props.overscan ?? 4,
    initialRect: {
      width: measuredRect().width,
      height: viewportHeight(),
    },
    observeElementRect: (_instance, notify) => {
      publishRect = notify;
      notify({ width: measuredRect().width, height: viewportHeight() });
      return () => {
        publishRect = undefined;
      };
    },
    observeElementOffset: (_instance, notify) => {
      publishOffset = notify;
      notify(0, false);
      return () => {
        publishOffset = undefined;
      };
    },
    scrollToFn: (offset) => scrollHandle?.scrollTo({ top: offset }),
    onChange: () => invalidate((value) => value + 1),
  });

  const virtualizer = new Virtualizer(options());
  const dispose = virtualizer._didMount();
  onCleanup(() => {
    if (scrollEndTimer !== undefined) clearTimeout(scrollEndTimer);
    resizeObserver?.disconnect();
    dispose();
  });

  const virtualItems = createMemo(() => {
    version();
    props.items();
    virtualizer.setOptions(options());
    virtualizer._willUpdate();
    return virtualizer.getVirtualItems();
  });
  const totalSize = createMemo(() => {
    virtualItems();
    return virtualizer.getTotalSize();
  });

  return (
    <view
      class={props.class}
      role={props.role}
      aria-label={props.accessibilityLabel}
      ref={(node) => {
        // Solid's published JSX types describe DOM nodes, while the universal
        // renderer supplies Wabou handles at runtime. Keep that conversion at
        // this renderer boundary instead of leaking DOM types into the core.
        scrollHandle = node as unknown as Handle;
        if (props.viewportHeight === undefined) {
          resizeObserver?.disconnect();
          resizeObserver = new ResizeObserver(([entry]) => {
            if (!entry) return;
            const rect = {
              width: entry.contentRect.width,
              height: entry.contentRect.height,
            };
            setMeasuredRect(rect);
            publishRect?.(rect);
          });
          resizeObserver.observe(node as never);
        }
        virtualizer._willUpdate();
      }}
      style={{
        overflow: "scroll",
        position: "relative",
        ...(props.viewportHeight === undefined
          ? {}
          : { height: `${props.viewportHeight}px` }),
        width: "100%",
      }}
      onScroll={(event) => {
        // Native scroll payloads expose logical offsets directly rather than
        // through HTMLElement.scrollTop/scrollLeft.
        const nativeEvent = event as unknown as ScrollEvent;
        lastOffset = nativeEvent.scrollY ?? 0;
        publishOffset?.(lastOffset, true);
        if (scrollEndTimer !== undefined) clearTimeout(scrollEndTimer);
        scrollEndTimer = setTimeout(() => {
          scrollEndTimer = undefined;
          publishOffset?.(lastOffset, false);
        }, 150);
      }}
    >
      <view
        style={{
          position: "relative",
          height: `${totalSize()}px`,
          width: "100%",
        }}
      >
        <For each={virtualItems()} keyed={false}>
          {(virtualItem) => {
            const index = () => virtualItem().index;
            const item = createVirtualRow(props.items, index);
            const identity = createVirtualItemIdentity(
              props.items,
              index,
              (_item, currentIndex) => itemKeys()[currentIndex] ?? currentIndex,
            );
            return (
              <view
                style={{
                  position: "absolute",
                  top: `${virtualItem().start}px`,
                  height: `${virtualItem().size}px`,
                  width: "100%",
                }}
              >
                <Show when={identity()} keyed>
                  {(_identity) =>
                    props.children(() => {
                      const current = item();
                      if (current === undefined)
                        throw new Error(
                          "VirtualList item disappeared while its row was mounted",
                        );
                      return current;
                    }, index)
                  }
                </Show>
              </view>
            );
          }}
        </For>
      </view>
    </view>
  );
}
