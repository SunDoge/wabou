import { Accessor } from "solid-js";
//#region src/primitives/scroll-reset.d.ts
export interface ScrollResetTarget {
  scrollTo(options: {
    left?: number;
    top?: number;
  }): void;
}
export interface ScrollResetOptions<K> {
  /** The explicitly owned viewport; no global/window fallback is used. */
  target: Accessor<ScrollResetTarget | undefined>;
  /** Reset whenever this navigation or content identity changes. */
  key: Accessor<K>;
  left?: number;
  top?: number;
}
/** Reset one explicitly selected native viewport after its key changes. */
export declare function createScrollReset<K>(options: ScrollResetOptions<K>): () => void;
//#endregion
//# sourceMappingURL=scroll-reset.d.mts.map