import { NativeWidgetProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/shader-layer.d.ts
export interface ShaderLayerConfig {
  /** WGSL defining `wabou_effect`, or a complete module when `module` is true. */
  source: string;
  /** Accept a complete single-pass module exposing `vs_main` and `fs_main`. */
  module?: boolean;
  /** Up to 256 scalar values exposed through `wabou.values` or binding zero. */
  values?: readonly number[];
  /** Initial animation time in seconds. */
  time?: number;
  /** Native animation clock multiplier. */
  speed?: number;
  /** Whether the native frame clock advances time. */
  animated?: boolean;
  /** Temporarily freeze the native frame clock. */
  paused?: boolean;
}
export interface ShaderLayerProps extends Omit<NativeWidgetProps<ShaderLayerConfig>, "tag" | "config" | "children">, ShaderLayerConfig {
  class?: string;
}
/** A layout-aware custom WGSL surface rendered and animated by the Vello host. */
export declare function ShaderLayer(props: ShaderLayerProps): JSX.Element;
//#endregion
//# sourceMappingURL=shader-layer.d.mts.map