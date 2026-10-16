import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ButtonProps } from "./button.mjs";
import { JSX } from "solid-js";
import { Handle, LayoutRect } from "@wabou/core/renderer";
//#region src/components/message-scroller.d.ts
export type MessageScrollDirection = "start" | "end";
export interface MessageScrollIntoViewOptions {
  margin?: number;
  align?: "nearest" | "start";
}
export declare function messageScrollRange(contentHeight: number, viewportHeight: number): number;
export declare function isMessageScrollNearEnd(scrollY: number, contentHeight: number, viewportHeight: number, threshold?: number): boolean;
/** Smallest vertical delta that reveals a target without disturbing visible content. */
export declare function messageScrollRevealDelta(viewport: LayoutRect, target: LayoutRect, margin?: number): number;
/** Vertical delta that places a target at the viewport's reading start. */
export declare function messageScrollStartDelta(viewport: LayoutRect, target: LayoutRect, margin?: number): number;
export interface MessageScrollerControls {
  followingEnd(): boolean;
  canScrollStart(): boolean;
  canScrollEnd(): boolean;
  activeAnchor(): string | undefined;
  scrollTo(direction: MessageScrollDirection): void;
  scrollIntoView(target: Handle, options?: MessageScrollIntoViewOptions): void;
  scrollToAnchor(anchor: string, options?: MessageScrollIntoViewOptions): void;
}
export declare function useMessageScroller(): MessageScrollerControls;
export interface MessageScrollerProps extends ViewProps {
  /** Start attached to the end of the conversation. Defaults to true. */
  followEnd?: boolean;
  /** Distance in logical pixels that still counts as being at the end. */
  endThreshold?: number;
}
export interface MessageAnchorRect {
  id: string;
  rect: LayoutRect;
}
/** Pick the last conversation anchor that has crossed the reading line. */
export declare function activeMessageAnchor(viewport: LayoutRect, anchors: readonly MessageAnchorRect[], offset?: number): string | undefined;
export declare function MessageScroller(props: MessageScrollerProps): JSX.Element;
export interface MessageScrollerViewportProps extends ViewProps {}
export declare function MessageScrollerViewport(props: MessageScrollerViewportProps): JSX.Element;
export declare function MessageScrollerContent(props: ViewProps): JSX.Element;
export interface MessageScrollerItemProps extends ViewProps {
  /** Stable semantic id used by conversation navigation. */
  anchor?: string;
}
export declare function MessageScrollerItem(props: MessageScrollerItemProps): JSX.Element;
export interface MessageScrollerButtonProps extends Omit<ButtonProps, "onClick"> {
  direction?: MessageScrollDirection;
  onClick?: ButtonProps["onClick"];
}
export declare function MessageScrollerButton(props: MessageScrollerButtonProps): JSX.Element;
export interface MessageScrollerNavigatorItem {
  id: string;
  label: string;
}
export interface MessageScrollerNavigatorProps {
  items: readonly MessageScrollerNavigatorItem[];
  "aria-label": string;
  itemAriaLabel(item: MessageScrollerNavigatorItem, index: number): string;
  minItems?: number;
  class?: string;
  railClass?: string;
}
/** Compact anchor rail for navigating long retained conversations. */
export declare function MessageScrollerNavigator(props: MessageScrollerNavigatorProps): JSX.Element;
//#endregion
//# sourceMappingURL=message-scroller.d.mts.map