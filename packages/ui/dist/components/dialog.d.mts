import { ModalControls, ModalProps } from "../primitives/modal.mjs";
import { ScrollAreaProps } from "../primitives/scroll-area.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/dialog.d.ts
export interface DialogProps extends Omit<ModalProps, "contentClass"> {
  contentClass?: string;
}
export declare function Dialog(props: DialogProps): JSX.Element;
export declare function DialogHeader(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function DialogFooter(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
/**
 * The shrinking, independently scrollable region between a dialog's fixed
 * header and footer. The dialog surface must have a bounded or maximum height.
 */
export interface DialogScrollBodyProps extends Omit<ScrollAreaProps, "class" | "contentClass"> {
  class?: string;
  contentClass?: string;
}
export declare function DialogScrollBody(props: DialogScrollBodyProps): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function DialogTitle(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function DialogDescription(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
export type { ModalControls as DialogControls };
//# sourceMappingURL=dialog.d.mts.map