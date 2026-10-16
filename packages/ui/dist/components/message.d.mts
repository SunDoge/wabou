import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/message.d.ts
export type MessageAlign = "start" | "end";
export type BubbleVariant = "default" | "secondary" | "muted" | "tinted" | "outline" | "ghost" | "destructive";
export declare function MessageGroup(props: ViewProps): JSX.Element;
export interface MessageProps extends Omit<ViewProps, "class"> {
  align?: MessageAlign;
  class?: string;
}
export declare function messageClass(align?: MessageAlign, className?: string): string;
export declare function Message(props: MessageProps): JSX.Element;
export declare function MessageAvatar(props: ViewProps): JSX.Element;
export declare function MessageContent(props: ViewProps): JSX.Element;
export declare function MessageHeader(props: TextProps): JSX.Element;
export declare const MessageFooter: typeof MessageHeader;
export declare function messageActionsClass(align?: MessageAlign, className?: string): string;
export interface MessageActionsProps extends Omit<ViewProps, "class"> {
  /** Override the containing message direction for a local action rail. */
  align?: MessageAlign;
  /** Keep actions visible, or reveal them while their message is hovered/focused. */
  visibility?: "always" | "interaction";
  class?: string;
}
/** Compact, consistently aligned actions belonging to one message. */
export declare function MessageActions(props: MessageActionsProps): JSX.Element;
export declare function BubbleGroup(props: ViewProps): JSX.Element;
export interface BubbleProps extends Omit<ViewProps, "class"> {
  variant?: BubbleVariant;
  align?: MessageAlign;
  class?: string;
}
export declare function bubbleClass(variant?: BubbleVariant, align?: MessageAlign, className?: string): string;
export declare function Bubble(props: BubbleProps): JSX.Element;
export declare function bubbleContentClass(variant: BubbleVariant, className?: string): string;
export declare function BubbleContent(props: ViewProps): JSX.Element;
export declare function BubbleReactions(props: {
  children?: JSX.Element;
  side?: "top" | "bottom";
  align?: MessageAlign;
  class?: string;
}): JSX.Element;
export type MarkerVariant = "default" | "separator" | "border";
export declare function Marker(props: {
  children?: JSX.Element;
  variant?: MarkerVariant;
  class?: string;
}): JSX.Element;
export declare function MarkerIcon(props: ViewProps): JSX.Element;
export declare function MarkerContent(props: TextProps): JSX.Element;
//#endregion
//# sourceMappingURL=message.d.mts.map