import { Button } from "./button.mjs";
import { Dialog, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "./dialog.mjs";
import { createComponent, createContext, useContext } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/alert-dialog.tsx
const AlertDialogContext = createContext();
function useAlertDialog() {
	const context = useContext(AlertDialogContext);
	if (!context) throw new Error("AlertDialog actions must be used inside AlertDialog");
	return context;
}
/**
* A blocking confirmation dialog. Backdrop dismissal is disabled by default so
* every close is an intentional cancel, confirmation, or Escape action.
*/
function AlertDialog(props) {
	return createComponent$1(Dialog, mergeProps(props, {
		contentRole: "alertdialog",
		get closeOnBackdrop() {
			return props.closeOnBackdrop ?? false;
		},
		children: (controls) => createComponent(AlertDialogContext, {
			value: controls,
			get children() {
				return typeof props.children === "function" ? props.children(controls) : props.children;
			}
		})
	}));
}
function closingHandler(close, handler) {
	return (event) => {
		handler?.(event);
		if (!event.defaultPrevented) close();
	};
}
function AlertDialogAction(props) {
	const dialog = useAlertDialog();
	return createComponent$1(Button, mergeProps(props, { get onClick() {
		return closingHandler(dialog.close, props.onClick);
	} }));
}
function AlertDialogCancel(props) {
	const dialog = useAlertDialog();
	return createComponent$1(Button, mergeProps(props, {
		get variant() {
			return props.variant ?? "outline";
		},
		get onClick() {
			return closingHandler(dialog.close, props.onClick);
		}
	}));
}
const AlertDialogHeader = DialogHeader;
const AlertDialogFooter = DialogFooter;
const AlertDialogTitle = DialogTitle;
const AlertDialogDescription = DialogDescription;
//#endregion
export { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle };

//# sourceMappingURL=alert-dialog.mjs.map