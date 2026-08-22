import { rgba } from "@wabou/core";
import type { JSX } from "solid-js";
import {
  Modal,
  type ModalControls,
  type ModalProps,
  Text,
  View,
} from "../primitives";
import { ScrollArea, type ScrollAreaProps } from "../primitives/scroll-area";
import { join } from "./class-names";
import { componentsElevation, useComponentsTheme } from "./theme";

export interface DialogProps extends Omit<ModalProps, "contentClass"> {
  contentClass?: string;
}

export function Dialog(props: DialogProps): JSX.Element {
  const theme = useComponentsTheme();
  return (
    <Modal
      {...props}
      motion={props.motion === undefined ? { fromScale: 0.98 } : props.motion}
      backdropClass={props.backdropClass}
      backdropStyle={{
        "background-color": rgba(0x00000033),
        ...props.backdropStyle,
      }}
      contentClass={join(
        "w-[480px] max-w-full min-w-0 flex flex-col gap-4 rounded-lg border border-subtle bg-surface p-5",
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

export function DialogHeader(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View class={join("flex flex-col gap-1", props.class)}>
      {props.children}
    </View>
  );
}

export function DialogFooter(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View class={join("flex items-center justify-end gap-2", props.class)}>
      {props.children}
    </View>
  );
}

/**
 * The shrinking, independently scrollable region between a dialog's fixed
 * header and footer. The dialog surface must have a bounded or maximum height.
 */
export interface DialogScrollBodyProps
  extends Omit<ScrollAreaProps, "class" | "contentClass"> {
  class?: string;
  contentClass?: string;
}

export function DialogScrollBody(props: DialogScrollBodyProps) {
  return (
    <ScrollArea
      {...props}
      class={join("min-h-0 flex-1", props.class)}
      contentClass={props.contentClass}
    >
      {props.children}
    </ScrollArea>
  );
}

export function DialogTitle(props: { children?: JSX.Element; class?: string }) {
  return (
    <Text class={join("text-lg font-semibold text-primary", props.class)}>
      {props.children}
    </Text>
  );
}

export function DialogDescription(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <Text
      class={join(
        "w-full min-w-0 whitespace-normal text-sm text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export type { ModalControls as DialogControls };
