import { Virtualizer, type VirtualizerOptions } from "@tanstack/virtual-core";
import {
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
  /** Visible viewport height in logical pixels. */
  viewportHeight: number;
  /** Extra rows rendered above/below the viewport. Defaults to 4. */
  overscan?: number;
  /** Explicit semantic role for the viewport, such as `listbox`. */
  role?: WabouSemanticRole;
  /** Accessible name for the native scroll viewport. */
  accessibilityLabel?: string;
  /** Render a single row given its item and absolute index. */
  children: (item: T, index: number) => JSX.Element;
}

interface ScrollEvent {
  scrollX?: number;
  scrollY?: number;
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
  let scrollEndTimer: ReturnType<typeof setTimeout> | undefined;
  let lastOffset = 0;
  const [version, invalidate] = createSignal(0, { equals: false });

  const options = (): VirtualizerOptions<Element, Element> => ({
    count: props.items().length,
    getScrollElement: () => (scrollHandle ? surface : null),
    estimateSize: () => props.itemHeight,
    overscan: props.overscan ?? 4,
    initialRect: { width: 0, height: props.viewportHeight },
    observeElementRect: (_instance, notify) => {
      notify({ width: 0, height: props.viewportHeight });
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
      role={props.role}
      aria-label={props.accessibilityLabel}
      ref={(node) => {
        // Solid's published JSX types describe DOM nodes, while the universal
        // renderer supplies Wabou handles at runtime. Keep that conversion at
        // this renderer boundary instead of leaking DOM types into the core.
        scrollHandle = node as unknown as Handle;
        virtualizer._willUpdate();
      }}
      style={{
        overflow: "scroll",
        position: "relative",
        height: `${props.viewportHeight}px`,
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
          {(virtualItem) => (
            <view
              style={{
                position: "absolute",
                top: `${virtualItem().start}px`,
                height: `${virtualItem().size}px`,
                width: "100%",
              }}
            >
              <Show when={virtualItem().index + 1} keyed>
                {(key) => {
                  const index = key - 1;
                  return props.children(props.items()[index]!, index);
                }}
              </Show>
            </view>
          )}
        </For>
      </view>
    </view>
  );
}
