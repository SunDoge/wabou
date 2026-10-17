import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ButtonProps } from "./button.mjs";
import { JSX } from "solid-js";
//#region src/components/attachment.d.ts
export type AttachmentState = "idle" | "uploading" | "processing" | "error" | "done";
export type AttachmentSize = "default" | "sm" | "xs";
export type AttachmentOrientation = "horizontal" | "vertical";
interface AttachmentContextValue {
  state(): AttachmentState;
  size(): AttachmentSize;
  orientation(): AttachmentOrientation;
}
export declare function attachmentClass(options: {
  state?: AttachmentState;
  size?: AttachmentSize;
  orientation?: AttachmentOrientation;
  class?: string;
}): string;
export interface AttachmentProps extends Omit<ViewProps, "class"> {
  state?: AttachmentState;
  size?: AttachmentSize;
  orientation?: AttachmentOrientation;
  class?: string;
}
/** File/task summary anatomy adapted from shadcn without DOM data selectors. */
export declare function Attachment(props: AttachmentProps): JSX.Element;
export declare function attachmentMediaClass(variant: "icon" | "image", context: Pick<AttachmentContextValue, "state" | "size" | "orientation">, className?: string): string;
export declare function AttachmentMedia(props: {
  children?: JSX.Element;
  variant?: "icon" | "image";
  class?: string;
}): JSX.Element;
export declare function AttachmentContent(props: ViewProps): JSX.Element;
export declare function AttachmentTitle(props: TextProps): JSX.Element;
export declare function AttachmentDescription(props: TextProps): JSX.Element;
export declare function AttachmentActions(props: ViewProps): JSX.Element;
export declare function AttachmentAction(props: ButtonProps): JSX.Element;
export declare function attachmentGroupClass(className?: string): string;
export declare function AttachmentGroup(props: ViewProps): JSX.Element;
//#endregion
export {};
//# sourceMappingURL=attachment.d.mts.map