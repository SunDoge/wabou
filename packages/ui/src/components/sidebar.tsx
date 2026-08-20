import { type JSX, omit } from "solid-js";
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
import { join } from "./class-names";
import { SearchField, type SearchFieldProps } from "./search-field";

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
}

/** Structural application sidebar. State, routing and width remain explicit. */
export function Sidebar(props: SidebarProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "group"}
      class={join(
        "h-full min-h-0 flex-none flex flex-col overflow-hidden bg-surface-muted",
        props.class,
      )}
    />
  );
}

export function SidebarHeader(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={join("flex-none border-b border-subtle bg-surface", props.class)}
    />
  );
}

export function SidebarSearch(props: SearchFieldProps): JSX.Element {
  const forwarded = omit(props, "class");
  return (
    <View class="flex-none p-2 border-b border-subtle bg-surface">
      <SearchField
        {...forwarded}
        placeholder={props.placeholder ?? "Search"}
        class={join("w-full", props.class)}
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
      class={join("min-h-0 flex-1", props.class)}
      contentClass={join("px-2 py-3", props.contentClass)}
    />
  );
}

export function SidebarGroup(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "group"}
      class={join("flex-none flex flex-col gap-0.5 mb-4", props.class)}
    />
  );
}

export function SidebarGroupLabel(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={join("px-2 py-1 text-xs font-medium text-muted", props.class)}
    />
  );
}

export interface SidebarMenuButtonProps
  extends Omit<HeadlessButtonProps, "class" | "unstyled"> {
  class?: string;
}

/** Consistent navigation row; applications still own activation and routing. */
export function SidebarMenuButton(props: SidebarMenuButtonProps): JSX.Element {
  const forwarded = omit(props, "class");
  return (
    <HeadlessButton
      {...forwarded}
      unstyled
      class={(state) =>
        join(
          "w-full min-w-0 h-8 px-3 justify-start gap-2 rounded-md text-sm",
          state.selected
            ? "bg-selected text-primary"
            : state.hovered
              ? "bg-control-hover text-primary"
              : "bg-transparent text-secondary",
          state.focusVisible && "border border-focus",
          props.class,
        )
      }
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
      class={join("px-3 py-6 flex flex-col items-center gap-1", props.class)}
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
      class={join("flex-none border-t border-subtle bg-surface", props.class)}
    />
  );
}
