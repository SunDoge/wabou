import { createComponent, createContext, type JSX, useContext } from "solid-js";
import type { ModalControls } from "../primitives";
import { Button, type ButtonProps } from "./button";
import {
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  type DialogProps,
  DialogTitle,
} from "./dialog";

interface AlertDialogContextValue {
  close(): void;
}

const AlertDialogContext = createContext<AlertDialogContextValue>();

function useAlertDialog(): AlertDialogContextValue {
  const context = useContext(AlertDialogContext);
  if (!context) {
    throw new Error("AlertDialog actions must be used inside AlertDialog");
  }
  return context;
}

export interface AlertDialogProps
  extends Omit<DialogProps, "children" | "contentRole"> {
  children?: JSX.Element | ((controls: ModalControls) => JSX.Element);
}

/**
 * A blocking confirmation dialog. Backdrop dismissal is disabled by default so
 * every close is an intentional cancel, confirmation, or Escape action.
 */
export function AlertDialog(props: AlertDialogProps): JSX.Element {
  return (
    <Dialog
      {...props}
      contentRole="alertdialog"
      closeOnBackdrop={props.closeOnBackdrop ?? false}
    >
      {(controls) =>
        createComponent(AlertDialogContext, {
          value: controls,
          get children() {
            return typeof props.children === "function"
              ? props.children(controls)
              : props.children;
          },
        })
      }
    </Dialog>
  );
}

function closingHandler(
  close: () => void,
  handler: ButtonProps["onClick"],
): NonNullable<ButtonProps["onClick"]> {
  return (event) => {
    handler?.(event);
    if (!event.defaultPrevented) close();
  };
}

export function AlertDialogAction(props: ButtonProps): JSX.Element {
  const dialog = useAlertDialog();
  return (
    <Button {...props} onClick={closingHandler(dialog.close, props.onClick)} />
  );
}

export function AlertDialogCancel(props: ButtonProps): JSX.Element {
  const dialog = useAlertDialog();
  return (
    <Button
      {...props}
      variant={props.variant ?? "outline"}
      onClick={closingHandler(dialog.close, props.onClick)}
    />
  );
}

export const AlertDialogHeader = DialogHeader;
export const AlertDialogFooter = DialogFooter;
export const AlertDialogTitle = DialogTitle;
export const AlertDialogDescription = DialogDescription;
