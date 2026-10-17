//#region src/primitives/hover.d.ts
export interface HoverResult {
  hovered: () => boolean;
  bindings: {
    onPointerEnter: () => void;
    onPointerLeave: () => void;
  };
}
/** Reactive hover state and event bindings for a single target. */
export declare function createHover(): HoverResult;
//#endregion
//# sourceMappingURL=hover.d.mts.map