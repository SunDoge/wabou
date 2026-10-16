import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { WabouStyle } from "@wabou/core/style";
import { JSX } from "solid-js";
//#region src/components/aspect-ratio.d.ts
export interface AspectRatioProps extends Omit<ViewProps, "style"> {
  /** Width divided by height. Defaults to a square. */
  ratio?: number;
  style?: WabouStyle;
}
export declare function aspectRatioStyle(ratio: number | undefined, style?: WabouStyle): WabouStyle;
/** A native Taffy aspect-ratio constraint with explicit overflow ownership. */
export declare function AspectRatio(props: AspectRatioProps): JSX.Element;
//#endregion
//# sourceMappingURL=aspect-ratio.d.mts.map