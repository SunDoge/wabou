import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { createFocusWithin, View, type ViewProps } from "../primitives";
import { Button, type ButtonProps } from "./button";
import { TextArea, type TextAreaProps } from "./input";

export interface PromptComposerProps extends Omit<ViewProps, "class"> {
  class?: string;
  disabled?: boolean;
  invalid?: boolean;
  /** Background utility owned by the compound surface. */
  surfaceClass?: string;
}

export interface PromptComposerRowProps extends Omit<ViewProps, "class"> {
  class?: string;
  /** Allow controls to form additional rows when the embedding surface opts in. */
  wrap?: boolean;
}

export function promptComposerEditorHeightClass(value: string): string {
  const lines = value.split("\n").length;
  if (lines >= 5 || value.length > 240) return "h-24";
  if (lines >= 3 || value.length > 120) return "h-20";
  if (lines >= 2 || value.length > 48) return "h-16";
  return "h-12";
}

export interface PromptComposerEditorProps extends TextAreaProps {
  value?: string;
}

/** Native multiline editor with density and chrome owned by PromptComposer. */
export function PromptComposerEditor(
  props: PromptComposerEditorProps,
): JSX.Element {
  return (
    <TextArea
      {...props}
      chrome="none"
      class={mergeClasses(
        "min-w-0 px-0 py-1",
        promptComposerEditorHeightClass(props.value ?? ""),
        props.class,
      )}
    />
  );
}

/** Stable circular primary action for a PromptComposer toolbar. */
export function PromptComposerAction(props: ButtonProps): JSX.Element {
  return (
    <Button
      {...props}
      variant={props.variant ?? "secondary"}
      size={props.size ?? "icon"}
      class={mergeClasses(
        "flex-none rounded-full border border-subtle",
        props.class,
      )}
    />
  );
}

export function promptComposerClass(
  focused: boolean,
  invalid: boolean,
  disabled: boolean,
  className?: string,
): string {
  return mergeClasses(
    "w-full min-w-0 rounded-xl border shadow-xs px-3 pt-2 pb-2 flex flex-col gap-2",
    invalid ? "border-danger" : focused ? "border-focus" : "border-subtle",
    disabled && "opacity-50",
    className,
  );
}

/** Shared compound surface for prompts, attachments, controls and status. */
export function PromptComposer(props: PromptComposerProps): JSX.Element {
  const focus = createFocusWithin();
  const forwarded = omit(
    props,
    "class",
    "children",
    "disabled",
    "invalid",
    "surfaceClass",
  );
  return (
    <View
      {...forwarded}
      {...focus.bindings}
      role={props.role ?? "group"}
      aria-disabled={props.disabled}
      aria-invalid={props.invalid}
      data-wabou-owns="surface focus-ring"
      class={mergeClasses(
        promptComposerClass(
          focus.focusWithin(),
          props.invalid ?? false,
          props.disabled ?? false,
          props.class,
        ),
        props.surfaceClass ?? "bg-input",
      )}
    >
      {props.children}
    </View>
  );
}

/** Compact metadata row above the authored prompt. */
export function PromptComposerStatus(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses(
        "w-full min-w-0 flex flex-row items-center justify-end gap-2",
        props.class,
      )}
    />
  );
}

/** Responsive action row below the authored prompt. */
export function PromptComposerToolbar(
  props: PromptComposerRowProps,
): JSX.Element {
  const forwarded = omit(props, "class", "wrap");
  return (
    <View
      {...forwarded}
      role={props.role ?? "toolbar"}
      class={mergeClasses(
        "w-full min-w-0 flex flex-row flex-nowrap items-center justify-between gap-1.5",
        props.wrap && "flex-wrap",
        props.class,
      )}
    />
  );
}

/** Shrink-safe group for the composer tools preceding its primary action. */
export function PromptComposerTools(
  props: PromptComposerRowProps,
): JSX.Element {
  const forwarded = omit(props, "class", "wrap");
  return (
    <View
      {...forwarded}
      role={props.role ?? "group"}
      class={mergeClasses(
        "min-w-0 flex-1 flex flex-row flex-nowrap items-center gap-0.5",
        props.wrap && "flex-wrap",
        props.class,
      )}
    />
  );
}
