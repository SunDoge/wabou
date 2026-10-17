import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/empty.d.ts
export type EmptyVariant = "surface" | "plain";
export type EmptyMediaVariant = "default" | "icon";
export interface EmptyProps extends Omit<ViewProps, "class"> {
  /** `plain` embeds inside an existing surface without nesting another card. */
  variant?: EmptyVariant;
  class?: string;
}
export declare function emptyClass(variant?: EmptyVariant, className?: string): string;
/** A composable empty-state region based on shadcn's Empty anatomy. */
export declare function Empty(props: EmptyProps): JSX.Element;
export declare function EmptyHeader(props: ViewProps): JSX.Element;
export interface EmptyMediaProps extends Omit<ViewProps, "class"> {
  variant?: EmptyMediaVariant;
  class?: string;
}
export declare function emptyMediaClass(variant?: EmptyMediaVariant, className?: string): string;
export declare function EmptyMedia(props: EmptyMediaProps): JSX.Element;
export declare function EmptyTitle(props: TextProps): JSX.Element;
export declare function EmptyDescription(props: TextProps): JSX.Element;
export declare function EmptyContent(props: ViewProps): JSX.Element;
//#endregion
//# sourceMappingURL=empty.d.mts.map