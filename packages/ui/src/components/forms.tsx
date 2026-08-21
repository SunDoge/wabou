import { For, type JSX, Show } from "solid-js";
import { match } from "ts-pattern";
import {
  createFocusWithin,
  TextArea as PrimitiveTextArea,
  type TextAreaProps as PrimitiveTextAreaProps,
  Text,
  View,
} from "../primitives";
import { Button, type ButtonProps } from "./button";
import { join } from "./class-names";
import { Input, type InputProps } from "./input";
export type FieldOrientation = "vertical" | "horizontal";
export function fieldClass(
  orientation: FieldOrientation = "vertical",
  invalid = false,
  className?: string,
): string {
  const layout = match(orientation)
    .with("vertical", () => "flex-col gap-2")
    .with("horizontal", () => "flex-row items-start gap-4")
    .exhaustive();
  return join(
    "w-full min-w-0 flex",
    layout,
    invalid && "text-danger-primary",
    className,
  );
}

export function Field(props: {
  children?: JSX.Element;
  orientation?: FieldOrientation;
  invalid?: boolean;
  class?: string;
}) {
  return (
    <View
      role="group"
      class={fieldClass(props.orientation, props.invalid ?? false, props.class)}
    >
      {props.children}
    </View>
  );
}

export function FieldSet(props: { children?: JSX.Element; class?: string }) {
  return (
    <View
      role="group"
      class={join("w-full min-w-0 flex flex-col gap-6", props.class)}
    >
      {props.children}
    </View>
  );
}

export function FieldLegend(props: {
  children?: JSX.Element;
  variant?: "legend" | "label";
  class?: string;
}) {
  return (
    <Text
      role="heading"
      class={join(
        "mb-1 font-medium text-primary",
        props.variant === "label" ? "text-sm" : "text-base",
        props.class,
      )}
    >
      {props.children}
    </Text>
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

export function FieldTitle(props: { children?: JSX.Element; class?: string }) {
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
export interface FieldErrorLike {
  message?: string;
}

export function uniqueFieldErrors(
  errors: ReadonlyArray<FieldErrorLike | undefined> | undefined,
): string[] {
  return [
    ...new Set(
      (errors ?? [])
        .map((error) => error?.message)
        .filter((message): message is string => Boolean(message)),
    ),
  ];
}

export function fieldErrorLabel(
  explicit: string | undefined,
  children: JSX.Element,
  messages: readonly string[],
): string | undefined {
  if (explicit) return explicit;
  if (typeof children === "string") return children;
  return messages.length > 0 ? messages.join(" ") : undefined;
}

export function FieldError(props: {
  children?: JSX.Element;
  errors?: ReadonlyArray<FieldErrorLike | undefined>;
  "aria-label"?: string;
  class?: string;
}) {
  const messages = () => uniqueFieldErrors(props.errors);
  const label = () =>
    fieldErrorLabel(props["aria-label"], props.children, messages());
  return (
    <View
      role="alert"
      aria-label={label()}
      class={join("w-full min-w-0 flex flex-col gap-1", props.class)}
    >
      <Show
        when={props.children !== undefined && props.children !== null}
        fallback={
          <For each={messages()}>
            {(message) => (
              <Text class="w-full min-w-0 whitespace-normal text-xs text-danger-primary">
                {message}
              </Text>
            )}
          </For>
        }
      >
        <Text class="w-full min-w-0 whitespace-normal text-xs text-danger-primary">
          {props.children}
        </Text>
      </Show>
    </View>
  );
}

export function FieldSeparator(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View
      class={join("w-full min-w-0 h-5 flex items-center gap-2", props.class)}
    >
      <View aria-hidden="true" class="flex-1 min-w-0 h-px bg-subtle" />
      <Show when={props.children !== undefined && props.children !== null}>
        <Text class="flex-none text-xs text-muted">{props.children}</Text>
      </Show>
      <View aria-hidden="true" class="flex-1 min-w-0 h-px bg-subtle" />
    </View>
  );
}
export function InputGroup(props: {
  children?: JSX.Element;
  class?: string;
  /** Background utility owned by the compound control. Defaults to `bg-input`. */
  surfaceClass?: string;
}) {
  const focus = createFocusWithin();
  return (
    <View
      {...focus.bindings}
      data-wabou-owns="surface focus-ring"
      class={join(
        "w-full h-8 flex items-center rounded-md border shadow-xs",
        props.surfaceClass ?? "bg-input",
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
      chrome="none"
      class={join("flex-1 min-w-0", props.class)}
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
