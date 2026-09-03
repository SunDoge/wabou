import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { Text, View, type ViewProps } from "../primitives";

export type SettingsItemOrientation = "horizontal" | "vertical";

export interface SettingsItemProps
  extends Omit<ViewProps, "children" | "class"> {
  title: string;
  description?: JSX.Element;
  children?: JSX.Element;
  orientation?: SettingsItemOrientation;
  disabled?: boolean;
  class?: string;
  labelClass?: string;
  controlClass?: string;
}

/**
 * One settings row with stable explanatory and control regions.
 * The row blocks its complete subtree when disabled; controls should also
 * receive `disabled` when they need to expose that state independently.
 */
export function SettingsItem(props: SettingsItemProps): JSX.Element {
  const orientation = () => props.orientation ?? "horizontal";
  const rest = omit(
    props,
    "title",
    "description",
    "children",
    "orientation",
    "disabled",
    "class",
    "labelClass",
    "controlClass",
    "style",
  );
  return (
    <View
      {...rest}
      role={props.role ?? "group"}
      aria-label={props["aria-label"] ?? props.title}
      aria-orientation={orientation()}
      aria-disabled={props.disabled}
      interactionBlocked={props.disabled || props.interactionBlocked}
      class={mergeClasses(
        "w-full min-w-0 flex",
        orientation() === "horizontal"
          ? "flex-row items-start justify-between gap-6"
          : "flex-col items-stretch gap-3",
        props.class,
      )}
      style={{ ...props.style, opacity: props.disabled ? 0.45 : undefined }}
    >
      <View
        class={mergeClasses(
          "min-w-0 flex-1 flex flex-col gap-1",
          props.labelClass,
        )}
      >
        <Text class="min-w-0 whitespace-normal text-sm font-medium text-primary">
          {props.title}
        </Text>
        {props.description !== undefined && props.description !== null ? (
          <Text class="min-w-0 whitespace-normal text-sm text-secondary">
            {props.description}
          </Text>
        ) : null}
      </View>
      <View
        class={mergeClasses(
          "min-w-0 flex items-center gap-2",
          orientation() === "horizontal"
            ? "flex-none justify-end"
            : "w-full justify-start",
          props.controlClass,
        )}
      >
        {props.children}
      </View>
    </View>
  );
}

export interface SettingsSectionProps {
  title: string;
  description?: string;
  children?: JSX.Element;
  /** Stack the explanation above the controls at a constrained viewport. */
  stacked?: boolean;
  class?: string;
  contentClass?: string;
}

/**
 * A settings-page section with one explanatory label and one control surface.
 * The page owns the responsive breakpoint and passes `stacked`; the component
 * owns the repeated alignment, spacing, and surface contract.
 */
export function SettingsSection(props: SettingsSectionProps): JSX.Element {
  return (
    <View
      role="group"
      aria-label={props.title}
      class={mergeClasses(
        "w-full min-w-0 flex items-start",
        props.stacked ? "flex-col gap-3" : "flex-row gap-7",
        props.class,
      )}
    >
      <View
        class={mergeClasses(
          "flex-none flex flex-col gap-1 pt-1",
          props.stacked ? "w-full" : "w-52",
        )}
      >
        <Text role="heading" class="text-base font-semibold text-primary">
          {props.title}
        </Text>
        {props.description ? (
          <Text class="text-sm text-secondary whitespace-normal">
            {props.description}
          </Text>
        ) : null}
      </View>
      <View
        class={mergeClasses(
          "w-full min-w-0 flex-1 flex flex-col gap-5 rounded-xl border border-subtle bg-surface px-5 py-5 shadow-xs",
          props.contentClass,
        )}
      >
        {props.children}
      </View>
    </View>
  );
}

export interface SettingsGroupProps {
  title: string;
  description?: string;
  children?: JSX.Element;
  class?: string;
}

/** A titled field group inside a SettingsSection control surface. */
export function SettingsGroup(props: SettingsGroupProps): JSX.Element {
  return (
    <View
      role="group"
      aria-label={props.title}
      class={mergeClasses("min-w-0 flex flex-col gap-4", props.class)}
    >
      <View class="min-w-0 flex flex-col gap-1">
        <Text class="text-sm font-semibold text-primary">{props.title}</Text>
        {props.description ? (
          <Text class="text-sm text-secondary whitespace-normal">
            {props.description}
          </Text>
        ) : null}
      </View>
      <View class="min-w-0 flex flex-col gap-4">{props.children}</View>
    </View>
  );
}
