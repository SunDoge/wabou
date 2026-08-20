import type { Handle } from "@wabou/core/renderer";
import { type Accessor, createSignal, onCleanup } from "solid-js";

export interface MeasuredSize {
  ref(node: Handle): void;
  width: Accessor<number>;
  height: Accessor<number>;
  measured: Accessor<boolean>;
}

export interface MeasuredSizeOptions {
  onChange?: (size: { width: number; height: number }) => void;
}

/** Inclusive logical-pixel constraints evaluated against a host node's content box. */
export interface ContainerSizeQuery {
  minWidth?: number;
  maxWidth?: number;
  minHeight?: number;
  maxHeight?: number;
}

export interface ContainerMatch extends MeasuredSize {
  matches: Accessor<boolean>;
}

function validateSizeQuery(query: ContainerSizeQuery): void {
  const entries = [
    ["minWidth", query.minWidth],
    ["maxWidth", query.maxWidth],
    ["minHeight", query.minHeight],
    ["maxHeight", query.maxHeight],
  ] as const;
  for (const [name, value] of entries) {
    if (value !== undefined && (!Number.isFinite(value) || value < 0))
      throw new RangeError(`${name} must be a finite non-negative number`);
  }
  if (
    query.minWidth !== undefined &&
    query.maxWidth !== undefined &&
    query.minWidth > query.maxWidth
  )
    throw new RangeError("minWidth cannot exceed maxWidth");
  if (
    query.minHeight !== undefined &&
    query.maxHeight !== undefined &&
    query.minHeight > query.maxHeight
  )
    throw new RangeError("minHeight cannot exceed maxHeight");
}

/** Observe the completed native content-box size of a host node. */
export function createMeasuredSize(
  options: MeasuredSizeOptions = {},
): MeasuredSize {
  const [width, setWidth] = createSignal(0);
  const [height, setHeight] = createSignal(0);
  const [measured, setMeasured] = createSignal(false);
  let observer: ResizeObserver | undefined;

  const ref = (node: Handle) => {
    observer?.disconnect();
    observer = new ResizeObserver(([entry]) => {
      if (!entry) return;
      const size = {
        width: entry.contentRect.width,
        height: entry.contentRect.height,
      };
      setWidth(size.width);
      setHeight(size.height);
      setMeasured(true);
      options.onChange?.(size);
    });
    observer.observe(node as never);
  };

  onCleanup(() => observer?.disconnect());
  return { ref, width, height, measured };
}

/**
 * Match constraints against a component's completed native content-box size.
 * The result remains false until the first measurement, avoiding a compact
 * layout flash during boot.
 */
export function createContainerMatch(
  query: ContainerSizeQuery,
  options: MeasuredSizeOptions = {},
): ContainerMatch {
  validateSizeQuery(query);
  const size = createMeasuredSize(options);
  const matches = () =>
    size.measured() &&
    (query.minWidth === undefined || size.width() >= query.minWidth) &&
    (query.maxWidth === undefined || size.width() <= query.maxWidth) &&
    (query.minHeight === undefined || size.height() >= query.minHeight) &&
    (query.maxHeight === undefined || size.height() <= query.maxHeight);
  return { ...size, matches };
}
