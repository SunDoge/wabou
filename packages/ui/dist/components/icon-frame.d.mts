import { IconProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ComponentTone } from "./tone.mjs";
//#region src/components/icon-frame.d.ts
export type IconFrameSize = "sm" | "default" | "lg" | "xl";
export type IconFrameVariant = "plain" | "soft" | "solid";
export interface IconFrameProps extends Omit<ViewProps, "children">, Pick<IconProps, "source" | "fill" | "label"> {
  /** Size of the square visual container. */
  size?: IconFrameSize;
  /** Explicit icon size. Defaults are tuned for each container size. */
  iconSize?: number | string;
  /** Color intent, independent from the frame's visual emphasis. */
  tone?: ComponentTone;
  variant?: IconFrameVariant;
  iconClass?: string;
}
/**
 * A square icon surface whose geometry is correct by construction.
 *
 * Use this for standalone icon tiles. Buttons and menu items already own their
 * icon alignment and should continue to use `Icon` directly.
 */
export declare function IconFrame(props: IconFrameProps): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
//# sourceMappingURL=icon-frame.d.mts.map