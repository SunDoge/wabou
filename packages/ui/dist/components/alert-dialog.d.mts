import { ModalControls } from "../primitives/modal.mjs";
import "../primitives.mjs";
import { ButtonProps } from "./button.mjs";
import { DialogDescription, DialogFooter, DialogHeader, DialogProps, DialogTitle } from "./dialog.mjs";
import { JSX } from "solid-js";
//#region src/components/alert-dialog.d.ts
export interface AlertDialogProps extends Omit<DialogProps, "children" | "contentRole"> {
  children?: JSX.Element | ((controls: ModalControls) => JSX.Element);
}
/**
 * A blocking confirmation dialog. Backdrop dismissal is disabled by default so
 * every close is an intentional cancel, confirmation, or Escape action.
 */
export declare function AlertDialog(props: AlertDialogProps): JSX.Element;
export declare function AlertDialogAction(props: ButtonProps): JSX.Element;
export declare function AlertDialogCancel(props: ButtonProps): JSX.Element;
export declare const AlertDialogHeader: typeof DialogHeader;
export declare const AlertDialogFooter: typeof DialogFooter;
export declare const AlertDialogTitle: typeof DialogTitle;
export declare const AlertDialogDescription: typeof DialogDescription;
//#endregion
//# sourceMappingURL=alert-dialog.d.mts.map