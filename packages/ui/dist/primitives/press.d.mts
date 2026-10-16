import { Accessor } from "solid-js";
//#region src/primitives/press.d.ts
export interface PressOptions {
  disabled?: Accessor<boolean> | boolean;
  onPress?: (event: unknown) => void;
}
export interface PressResult {
  pressed: () => boolean;
  bindings: {
    onPointerDown: () => void;
    onPointerUp: () => void;
    onPointerCancel: () => void;
    onPointerLeave: () => void;
    onClick: (event: unknown) => void;
  };
}
/** Reactive pointer-press state with disabled-aware activation. */
export declare function createPress(options?: PressOptions): PressResult;
export interface ActiveResult {
  active: () => boolean;
  bindings: PressResult["bindings"];
}
/** CSS `:active`-like state without an activation callback. */
export declare function createActive(disabled?: Accessor<boolean> | boolean): ActiveResult;
//#endregion
//# sourceMappingURL=press.d.mts.map