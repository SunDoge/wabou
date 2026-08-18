import { rgba } from "@wabou/core";
import {
  Modal,
  type ModalControls,
  type ModalProps,
  Text,
  View,
} from "@wabou/primitives";
import type { JSX } from "solid-js";
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
