import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { createFocusWithin, View, type ViewProps } from "../primitives";

export interface PromptComposerProps extends Omit<ViewProps, "class"> {
  class?: string;
  disabled?: boolean;
  invalid?: boolean;
  /** Background utility owned by the compound surface. */
  surfaceClass?: string;
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
export function PromptComposerToolbar(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "toolbar"}
      class={mergeClasses(
        "w-full min-w-0 flex flex-row flex-wrap items-center justify-between gap-1.5",
        props.class,
      )}
    />
  );
}

/** Shrink-safe group for the composer tools preceding its primary action. */
export function PromptComposerTools(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "group"}
      class={mergeClasses(
        "min-w-0 flex-1 flex flex-row flex-wrap items-center gap-0.5",
        props.class,
      )}
    />
  );
}
