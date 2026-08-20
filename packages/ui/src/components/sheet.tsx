import { rgba } from "@wabou/core";
import type { JSX } from "solid-js";
import { match } from "ts-pattern";
import { Modal, type ModalMotionOptions, type ModalProps } from "../primitives";
import { join } from "./class-names";
import {
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogScrollBody,
  DialogTitle,
} from "./dialog";
import { componentsElevation, useComponentsTheme } from "./theme";

export type SheetSide = "top" | "right" | "bottom" | "left";

export interface SheetProps extends Omit<ModalProps, "contentClass"> {
  side?: SheetSide;
  contentClass?: string;
}

const geometry = (side: SheetSide) =>
  match(side)
    .with("left", () => ({
      backdrop: { "align-items": "stretch", "justify-content": "flex-start" },
      content: "h-full w-[400px] max-w-full border-r",
    }))
    .with("right", () => ({
      backdrop: { "align-items": "stretch", "justify-content": "flex-end" },
      content: "h-full w-[400px] max-w-full border-l",
    }))
    .with("top", () => ({
      backdrop: { "align-items": "flex-start", "justify-content": "stretch" },
      content: "w-full max-h-[80%] border-b",
    }))
    .with("bottom", () => ({
      backdrop: { "align-items": "flex-end", "justify-content": "stretch" },
      content: "w-full max-h-[80%] border-t",
    }))
    .exhaustive();

const sheetMotion = (side: SheetSide): ModalMotionOptions =>
  match(side)
    .with("left", () => ({ duration: 0.18, fromX: -32 }))
    .with("right", () => ({ duration: 0.18, fromX: 32 }))
    .with("top", () => ({ duration: 0.18, fromY: -32 }))
    .with("bottom", () => ({ duration: 0.18, fromY: 32 }))
    .exhaustive();

/** A modal edge panel that shares native focus isolation with Dialog. */
export function Sheet(props: SheetProps): JSX.Element {
  const theme = useComponentsTheme();
  const side = () => props.side ?? "right";
  const placement = () => geometry(side());
  return (
    <Modal
      {...props}
      motion={props.motion === undefined ? sheetMotion(side()) : props.motion}
      backdropStyle={{
        "background-color": rgba(0x00000033),
        ...placement().backdrop,
        ...props.backdropStyle,
      }}
      contentClass={join(
        "min-w-0 min-h-0 flex flex-col gap-4 border-subtle bg-surface p-5",
        placement().content,
        props.contentClass,
      )}
      contentShadows={
        props.contentShadows === undefined
          ? componentsElevation(theme(), "modal")
          : props.contentShadows
      }
    />
  );
}

export {
  DialogDescription as SheetDescription,
  DialogFooter as SheetFooter,
  DialogHeader as SheetHeader,
  DialogScrollBody as SheetScrollBody,
  DialogTitle as SheetTitle,
};
