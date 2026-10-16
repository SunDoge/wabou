import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/card.d.ts
export type CardVariant = "raised" | "filled" | "outline" | "plain";
export type CardSize = "sm" | "default" | "lg";
export interface CardProps extends Omit<ViewProps, "class"> {
  variant?: CardVariant;
  size?: CardSize;
  class?: string;
}
export declare function Card(props: CardProps): JSX.Element;
export declare function CardHeader(props: ViewProps): JSX.Element;
export declare function CardTitle(props: TextProps): JSX.Element;
export declare function CardDescription(props: TextProps): JSX.Element;
/** Top-end action slot owned by the relative CardHeader surface. */
export declare function CardAction(props: ViewProps): JSX.Element;
export declare function CardContent(props: ViewProps): JSX.Element;
export declare function CardFooter(props: ViewProps): JSX.Element;
//#endregion
//# sourceMappingURL=card.d.mts.map