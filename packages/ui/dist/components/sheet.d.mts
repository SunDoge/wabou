import { ModalProps } from "../primitives/modal.mjs";
import "../primitives.mjs";
import { DialogDescription, DialogFooter, DialogHeader, DialogScrollBody, DialogTitle } from "./dialog.mjs";
import { JSX } from "solid-js";
//#region src/components/sheet.d.ts
export type SheetSide = "top" | "right" | "bottom" | "left";
export interface SheetProps extends Omit<ModalProps, "contentClass"> {
  side?: SheetSide;
  contentClass?: string;
}
/** A modal edge panel that shares native focus isolation with Dialog. */
export declare function Sheet(props: SheetProps): JSX.Element;
//#endregion
export { DialogDescription as SheetDescription, DialogFooter as SheetFooter, DialogHeader as SheetHeader, DialogScrollBody as SheetScrollBody, DialogTitle as SheetTitle };
//# sourceMappingURL=sheet.d.mts.map