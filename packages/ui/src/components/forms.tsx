import type { Handle } from "@wabou/core/renderer";
import { mergeClasses } from "@wabou/core/style";
import {
  createContext,
  For as ForValue,
  type JSX,
  omit,
  Show,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import {
  createFocusWithin,
  TextArea as PrimitiveTextArea,
  type TextAreaProps as PrimitiveTextAreaProps,
  Text,
  View,
  type ViewProps,
} from "../primitives";
import { Button, type ButtonProps } from "./button";
import { Input, type InputProps } from "./input";
import { Label, type LabelProps } from "./label";
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
  return mergeClasses(
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
      class={mergeClasses("w-full min-w-0 flex flex-col gap-6", props.class)}
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
      class={mergeClasses(
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
    <View class={mergeClasses("flex flex-col gap-4", props.class)}>
      {props.children}
    </View>
  );
}
export interface FieldLabelProps extends LabelProps {}

export function FieldLabel(props: FieldLabelProps) {
  return <Label {...props} />;
}

export interface LabeledFieldProps {
  label: JSX.Element;
  description?: JSX.Element;
  invalid?: boolean;
  disabled?: boolean;
  errors?: ReadonlyArray<FieldErrorLike | undefined>;
  class?: string;
  /** Render the native control and attach the supplied ref to its focus owner. */
  renderControl: (ref: (node: Handle) => void) => JSX.Element;
}

/**
 * A complete native field whose visible label always focuses its control.
 * This avoids repeating ad-hoc Handle plumbing in every settings surface.
 */
export function LabeledField(props: LabeledFieldProps) {
  let control: Handle | undefined;
  const errors = () => uniqueFieldErrors(props.errors);
  return (
    <Field invalid={props.invalid ?? errors().length > 0} class={props.class}>
      <FieldLabel disabled={props.disabled} control={() => control}>
        {props.label}
      </FieldLabel>
      {props.renderControl((node) => {
        control = node;
      })}
      <Show
        when={props.description !== undefined && props.description !== null}
      >
        <FieldDescription>{props.description}</FieldDescription>
      </Show>
      <Show when={errors().length > 0}>
        <FieldError errors={props.errors} />
      </Show>
    </Field>
  );
}

export function FieldTitle(props: { children?: JSX.Element; class?: string }) {
  return (
    <Text class={mergeClasses("text-sm font-medium text-primary", props.class)}>
      {props.children}
    </Text>
  );
}
export function FieldContent(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View
      class={mergeClasses("min-w-0 flex-1 flex flex-col gap-1", props.class)}
    >
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
      class={mergeClasses(
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
      class={mergeClasses("w-full min-w-0 flex flex-col gap-1", props.class)}
    >
      <Show
        when={props.children !== undefined && props.children !== null}
        fallback={
          <ForValue each={messages()}>
            {(message) => (
              <Text class="w-full min-w-0 whitespace-normal text-xs text-danger-primary">
                {message}
              </Text>
            )}
          </ForValue>
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
      class={mergeClasses(
        "w-full min-w-0 h-5 flex items-center gap-2",
        props.class,
      )}
    >
      <View aria-hidden="true" class="flex-1 min-w-0 h-px bg-subtle" />
      <Show when={props.children !== undefined && props.children !== null}>
        <Text class="flex-none text-xs text-muted">{props.children}</Text>
      </Show>
      <View aria-hidden="true" class="flex-1 min-w-0 h-px bg-subtle" />
    </View>
  );
}
export type InputGroupOrientation = "horizontal" | "vertical";
export type InputGroupVariant = "default" | "quiet";
export type InputGroupAddonAlign =
  | "inline-start"
  | "inline-end"
  | "block-start"
  | "block-end";

interface InputGroupContextValue {
  registerControl(node: Handle): void;
  focusControl(): void;
}

const InputGroupContext = createContext<InputGroupContextValue>();

function useInputGroup(): InputGroupContextValue | undefined {
  return useContext(InputGroupContext);
}

export function inputGroupClass(
  orientation: InputGroupOrientation,
  focused: boolean,
  invalid: boolean,
  variant: InputGroupVariant = "default",
): string {
  return mergeClasses(
    "relative w-full min-w-0 flex rounded-lg border",
    variant === "default" ? "shadow-xs" : "shadow-none",
    orientation === "horizontal"
      ? "h-8 flex-row items-center"
      : "h-auto flex-col items-stretch",
    invalid
      ? "border-danger"
      : focused
        ? "border-focus"
        : variant === "quiet"
          ? "border-transparent"
          : "border-strong",
  );
}

export interface InputGroupProps extends Omit<ViewProps, "children"> {
  children?: JSX.Element;
  orientation?: InputGroupOrientation;
  variant?: InputGroupVariant;
  invalid?: boolean;
  disabled?: boolean;
  /** Background utility owned by the compound control. Defaults to `bg-input`. */
  surfaceClass?: string;
}

export function InputGroup(props: InputGroupProps) {
  const focus = createFocusWithin();
  let control: Handle | undefined;
  const context: InputGroupContextValue = {
    registerControl(node) {
      control = node;
    },
    focusControl() {
      if (!props.disabled) control?.focus();
    },
  };
  const forwarded = omit(
    props,
    "children",
    "orientation",
    "variant",
    "invalid",
    "disabled",
    "surfaceClass",
    "class",
  );
  return (
    <InputGroupContext value={context}>
      <View
        {...forwarded}
        {...focus.bindings}
        role={props.role ?? "group"}
        aria-invalid={props.invalid}
        aria-disabled={props.disabled}
        data-wabou-owns="surface focus-ring"
        class={mergeClasses(
          inputGroupClass(
            props.orientation ?? "horizontal",
            focus.focusWithin(),
            props.invalid ?? false,
            props.variant ?? "default",
          ),
          props.surfaceClass ??
            (props.variant === "quiet" ? "bg-transparent" : "bg-input"),
          props.disabled && "opacity-50",
          props.class,
        )}
      >
        {props.children}
      </View>
    </InputGroupContext>
  );
}
export function InputGroupInput(props: InputProps) {
  const group = useInputGroup();
  return (
    <Input
      {...props}
      ref={(node) => {
        group?.registerControl(node);
        props.ref?.(node);
      }}
      chrome="none"
      class={mergeClasses("h-full flex-1 min-w-0", props.class)}
    />
  );
}

export interface InputGroupAddonProps extends ViewProps {
  align?: InputGroupAddonAlign;
  focusControl?: boolean;
}

export function inputGroupAddonClass(align: InputGroupAddonAlign): string {
  return match(align)
    .with(
      "inline-start",
      "inline-end",
      () =>
        "h-full flex-none px-3 flex items-center justify-center gap-2 text-sm text-muted",
    )
    .with(
      "block-start",
      "block-end",
      () =>
        "w-full flex-none px-3 py-2 flex items-center justify-start gap-2 text-sm text-muted",
    )
    .exhaustive();
}

export function InputGroupAddon(props: InputGroupAddonProps) {
  const group = useInputGroup();
  const forwarded = omit(props, "align", "focusControl", "class", "onClick");
  return (
    <View
      {...forwarded}
      role={props.role ?? "group"}
      class={mergeClasses(
        inputGroupAddonClass(props.align ?? "inline-start"),
        props.class,
      )}
      onClick={(event) => {
        if (props.focusControl ?? true) group?.focusControl();
        props.onClick?.(event);
      }}
    />
  );
}

export function InputGroupText(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <Text class={mergeClasses("flex-none text-sm text-muted", props.class)}>
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
      class={mergeClasses("mx-1", props.class)}
    />
  );
}
export function InputGroupTextArea(
  props: PrimitiveTextAreaProps & { class?: string },
) {
  const group = useInputGroup();
  return (
    <PrimitiveTextArea
      {...props}
      ref={(node) => {
        group?.registerControl(node);
        props.ref?.(node);
      }}
      data-wabou-owns="native-editor"
      class={mergeClasses(
        "w-full h-24 px-3 py-2 text-sm text-primary",
        props.class,
      )}
    />
  );
}
