import { mergeClasses } from "@wabou/core/style";
import {
  createComponent,
  createContext,
  type JSX,
  omit,
  useContext,
} from "solid-js";
import {
  Button as HeadlessButton,
  type ButtonProps as HeadlessButtonProps,
  ScrollArea,
  type ScrollAreaProps,
  Text,
  type TextProps,
  View,
  type ViewProps,
} from "../primitives";
import { SearchField, type SearchFieldProps } from "./search-field";
import { createControllableState } from "./state";
import {
  type ComponentsElevation,
  componentsElevation,
  useComponentsTheme,
} from "./theme";
import { workbenchHeaderClass } from "./workbench-style";

interface SidebarMenuContextValue {
  managed: boolean;
  value(): string | undefined;
  select(value: string): void;
}

const SidebarMenuContext = createContext<SidebarMenuContextValue>({
  managed: false,
  value: () => undefined,
  select: () => {},
});

export interface SidebarSearchGroup<Item> {
  label: string;
  items: readonly Item[];
}

/**
 * Filter grouped sidebar data without taking ownership of routing or identity.
 * Group labels participate in matching so a query can reveal a whole section.
 */
export function filterSidebarGroups<Item>(
  groups: readonly SidebarSearchGroup<Item>[],
  query: string,
  searchableText: (item: Item) => string,
): SidebarSearchGroup<Item>[] {
  const needle = query.trim().toLowerCase();
  if (!needle)
    return groups.map((group) => ({ ...group, items: [...group.items] }));

  return groups.flatMap((group) => {
    const groupMatches = group.label.toLowerCase().includes(needle);
    const items = groupMatches
      ? [...group.items]
      : group.items.filter((item) =>
          searchableText(item).toLowerCase().includes(needle),
        );
    return items.length === 0 ? [] : [{ ...group, items }];
  });
}

export interface SidebarProps extends Omit<ViewProps, "class"> {
  class?: string;
  /** Native shadow recipe for sidebars that float inside a window frame. */
  elevation?: ComponentsElevation;
}

/** Structural application sidebar. State, routing and width remain explicit. */
export function Sidebar(props: SidebarProps): JSX.Element {
  const theme = useComponentsTheme();
  const forwarded = omit(props, "class", "elevation", "shadows");
  return (
    <View
      {...forwarded}
      role={props.role ?? "group"}
      class={mergeClasses(
        "h-full min-h-0 flex-none flex flex-col overflow-hidden bg-surface-muted",
        props.class,
      )}
      shadows={
        props.shadows === undefined && props.elevation
          ? componentsElevation(theme(), props.elevation)
          : props.shadows
      }
    />
  );
}

export function SidebarHeader(props: ViewProps): JSX.Element {
  return <View {...props} class={workbenchHeaderClass(props.class)} />;
}

export function SidebarSearch(props: SearchFieldProps): JSX.Element {
  const forwarded = omit(props, "class");
  return (
    <View class="flex-none p-2 border-b border-subtle bg-surface">
      <SearchField
        {...forwarded}
        placeholder={props.placeholder ?? "Search"}
        class={mergeClasses("w-full", props.class)}
      />
    </View>
  );
}

export interface SidebarContentProps extends ScrollAreaProps {
  contentClass?: string;
}

/** The only scrolling region in a standard sidebar; header/footer stay fixed. */
export function SidebarContent(props: SidebarContentProps): JSX.Element {
  return (
    <ScrollArea
      {...props}
      class={mergeClasses("min-h-0 flex-1", props.class)}
      contentClass={mergeClasses("px-3 py-4", props.contentClass)}
    />
  );
}

export function SidebarGroup(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "group"}
      class={mergeClasses("flex-none flex flex-col gap-1 mb-5", props.class)}
    />
  );
}

export function SidebarGroupLabel(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={mergeClasses(
        "px-2 py-1.5 text-xs font-medium text-muted",
        props.class,
      )}
    />
  );
}

export interface SidebarMenuProps extends Omit<ViewProps, "class"> {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  class?: string;
}

/**
 * Single-selection navigation scope for sidebar destinations.
 * Buttons without a value remain actions and never become selected items.
 */
export function SidebarMenu(props: SidebarMenuProps): JSX.Element {
  const state = createControllableState<string | undefined>({
    value: () => props.value,
    defaultValue: props.defaultValue,
    onChange: (value) => value !== undefined && props.onValueChange?.(value),
  });
  const forwarded = omit(
    props,
    "value",
    "defaultValue",
    "onValueChange",
    "class",
    "children",
  );
  return createComponent(SidebarMenuContext, {
    value: {
      managed: true,
      value: state.value,
      select: (value) => state.set(value),
    },
    get children() {
      return (
        <View
          {...forwarded}
          role={props.role ?? "group"}
          class={mergeClasses("min-w-0 flex flex-col gap-1", props.class)}
        >
          {props.children}
        </View>
      );
    },
  });
}

export interface SidebarMenuButtonProps
  extends Omit<HeadlessButtonProps, "class" | "unstyled"> {
  /** Value controlled by the nearest SidebarMenu. Omit for action buttons. */
  value?: string;
  class?: string;
}

/** Consistent navigation row; applications still own activation and routing. */
export function SidebarMenuButton(props: SidebarMenuButtonProps): JSX.Element {
  const menu = useContext(SidebarMenuContext);
  const forwarded = omit(props, "class", "value", "selected", "onClick");
  const selected = () =>
    props.value !== undefined && menu.managed
      ? menu.value() === props.value
      : (props.selected ?? false);
  return (
    <HeadlessButton
      {...forwarded}
      unstyled
      selected={selected()}
      aria-selected={selected()}
      class={(state) =>
        mergeClasses(
          "w-full min-w-0 h-8 px-3 justify-start gap-2.5 rounded-lg text-sm",
          state.pressed
            ? "bg-control-pressed text-primary"
            : state.hovered
              ? "bg-control-hover text-primary"
              : state.selected
                ? "bg-selected text-primary"
                : "bg-transparent text-secondary",
          state.focusVisible && "border border-focus",
          props.class,
        )
      }
      onClick={(event) => {
        if (props.value !== undefined && menu.managed) menu.select(props.value);
        props.onClick?.(event);
      }}
    />
  );
}

export function SidebarEmpty(props: {
  title?: string;
  description?: string;
  class?: string;
}): JSX.Element {
  return (
    <View
      class={mergeClasses(
        "px-3 py-6 flex flex-col items-center gap-1",
        props.class,
      )}
    >
      <Text role="status" class="text-sm text-secondary">
        {props.title ?? "No results found"}
      </Text>
      {props.description ? (
        <Text class="text-xs text-muted">{props.description}</Text>
      ) : null}
    </View>
  );
}

export function SidebarFooter(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses(
        "flex-none border-t border-subtle bg-surface",
        props.class,
      )}
    />
  );
}
