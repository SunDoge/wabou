import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/direction.d.ts
export type Direction = "ltr" | "rtl";
export interface DirectionProviderProps {
  dir: Direction;
  children?: JSX.Element;
}
/** Own logical direction in JavaScript instead of relying on web inheritance. */
export declare function DirectionProvider(props: DirectionProviderProps): JSX.Element;
export declare function useDirection(): Direction;
export interface DirectionalRowProps extends Omit<ViewProps, "class"> {
  class?: string;
}
export declare function DirectionalRow(props: DirectionalRowProps): JSX.Element;
export interface DirectionalTextProps extends Omit<TextProps, "class"> {
  class?: string;
}
export declare function DirectionalText(props: DirectionalTextProps): JSX.Element;
//#endregion
//# sourceMappingURL=direction.d.mts.map