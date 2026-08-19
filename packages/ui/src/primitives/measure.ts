import type { Handle } from "@wabou/core/renderer";
import { createSignal, onCleanup, type Accessor } from "solid-js";

export interface MeasuredSize {
  ref(node: Handle): void;
  width: Accessor<number>;
  height: Accessor<number>;
  measured: Accessor<boolean>;
}

export interface MeasuredSizeOptions {
  onChange?: (size: { width: number; height: number }) => void;
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
