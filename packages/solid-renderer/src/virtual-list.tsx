// Windowed list — the SolidJS-as-DSL take on virtualization. Only the rows
// in (or near) the viewport are materialised as nodes; the reconciler
// (`<Index>`, keyed by slot) recycles slots as the visible range shifts, so
// off-screen rows are dropped and new ones created. Scroll position is driven
// by `wheel` events accumulated in a signal — no native scroll container, no
// DOM. The host's `overflow: hidden` clips; absolute `top` per slot places rows.

import { Index, type JSX } from "solid-js";
import { createMemo, createSignal } from "solid-js";

export interface VirtualListProps<T> {
  /** Accessor for the full backing array. Only the visible slice renders. */
  items: () => readonly T[];
  /** Fixed height of every row, in px. */
  itemHeight: number;
  /** Visible viewport height in px (the scrollable region's height). */
  viewportHeight: number;
  /** Extra rows rendered above/below the viewport. Defaults to 4. */
  overscan?: number;
  /** Render a single row given its item and absolute index. */
  children: (item: T, index: number) => JSX.Element;
}

export function VirtualList<T>(props: VirtualListProps<T>): JSX.Element {
  const [scrollTop, setScrollTop] = createSignal(0);
  const maxScroll = createMemo(() =>
    Math.max(0, props.items().length * props.itemHeight - props.viewportHeight),
  );
  const view = createMemo(() => {
    const st = scrollTop();
    const ih = props.itemHeight;
    const overscan = props.overscan ?? 4;
    const count = props.items().length;
    const start = Math.max(0, Math.floor(st / ih) - overscan);
    const end = Math.min(
      count,
      Math.ceil((st + props.viewportHeight) / ih) + overscan,
    );
    // Sub-row offset: how far the slot grid is translated up for partial rows.
    const partial = st - Math.floor(st / ih) * ih;
    return { start, end, partial };
  });
  const slice = createMemo(() => {
    const { start, end } = view();
    return props.items().slice(start, end);
  });
  const onWheel = (e: { deltaY?: number; preventDefault?: () => void }) => {
    // The host skips native scroll when a wheel listener is in the chain, so
    // this handler is the sole source of scroll position.
    e.preventDefault?.();
    const next = scrollTop() + (e.deltaY ?? 0);
    setScrollTop(Math.max(0, Math.min(next, maxScroll())));
  };

  return (
    <div
      style={{
        overflow: "hidden",
        position: "relative",
        height: `${props.viewportHeight}px`,
      }}
      onWheel={onWheel}
    >
      <Index each={slice()}>
        {(item, slot) => (
          <div
            style={{
              position: "absolute",
              top: `${slot * props.itemHeight - view().partial}px`,
              height: `${props.itemHeight}px`,
              width: "100%",
            }}
          >
            {props.children(item(), view().start + slot)}
          </div>
        )}
      </Index>
    </div>
  );
}
