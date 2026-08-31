import { mergeClasses } from "@wabou/core/style";
import type { JSX } from "solid-js";
import { Text, View } from "../primitives";

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
