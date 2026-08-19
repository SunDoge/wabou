import {
  createFocusWithin,
  TextArea as PrimitiveTextArea,
  type TextAreaProps as PrimitiveTextAreaProps,
  Text,
  View,
} from "../primitives";
import type { JSX } from "solid-js";
import { match } from "ts-pattern";
import { join } from "./class-names";
import { Button, type ButtonProps, Input, type InputProps } from "./index";
export type FieldOrientation = "vertical" | "horizontal";
export function Field(props: {
  children?: JSX.Element;
  orientation?: FieldOrientation;
  invalid?: boolean;
  class?: string;
}) {
  const layout = () =>
    match(props.orientation ?? "vertical")
      .with("vertical", () => "flex-col gap-2")
      .with("horizontal", () => "flex-row items-start gap-4")
      .exhaustive();
  return (
    <View
      class={join(
        "w-full flex",
        layout(),
        props.invalid && "text-danger-primary",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}
export function FieldGroup(props: { children?: JSX.Element; class?: string }) {
  return (
    <View class={join("flex flex-col gap-5", props.class)}>
      {props.children}
    </View>
  );
}
export function FieldLabel(props: { children?: JSX.Element; class?: string }) {
  return (
    <Text class={join("text-sm font-medium text-primary", props.class)}>
      {props.children}
    </Text>
  );
}
export function FieldContent(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View class={join("min-w-0 flex-1 flex flex-col gap-1", props.class)}>
      {props.children}
    </View>
  );
}
export function FieldDescription(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <Text
      class={join(
        "w-full min-w-0 whitespace-normal text-xs text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}
export function FieldError(props: { children?: JSX.Element; class?: string }) {
  return (
    <Text
      role="alert"
      class={join(
        "w-full min-w-0 whitespace-normal text-xs text-danger-primary",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}
export function InputGroup(props: { children?: JSX.Element; class?: string }) {
  const focus = createFocusWithin();
  return (
    <View
      {...focus.bindings}
      class={join(
        "w-full h-8 flex items-center rounded-md border bg-input shadow-xs",
        focus.focusWithin() ? "border-focus" : "border-strong",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}
export function InputGroupInput(props: InputProps) {
  return (
    <Input
      {...props}
      class={join(
        "flex-1 min-w-0 border-transparent bg-transparent",
        props.class,
      )}
    />
  );
}
export function InputGroupText(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <Text class={join("flex-none px-3 text-sm text-muted", props.class)}>
      {props.children}
    </Text>
  );
}
export function InputGroupButton(props: ButtonProps) {
  return (
    <Button
      {...props}
      size={props.size ?? "sm"}
      variant={props.variant ?? "ghost"}
      class={join("mx-1", props.class)}
    />
  );
}
export function InputGroupTextArea(
  props: PrimitiveTextAreaProps & { class?: string },
) {
  return (
    <PrimitiveTextArea
      {...props}
      class={join(
        "w-full h-24 px-3 py-2 border-transparent bg-transparent text-sm",
        props.class,
      )}
    />
  );
}
