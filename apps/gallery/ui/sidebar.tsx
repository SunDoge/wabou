import {
  Badge,
  filterSidebarGroups,
  Sidebar,
  SidebarContent,
  SidebarEmpty,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenuButton,
  SidebarSearch,
  Text,
  View,
} from "@wabou/ui";
import { createMemo, createSignal, For as ForValue, Show } from "solid-js";

export interface SidebarItem<Id extends string = string> {
  id: Id;
  name: string;
}

export interface GallerySidebarGroup<Id extends string = string> {
  label: string;
  items: readonly SidebarItem<Id>[];
}

export interface GallerySidebarProps<Id extends string> {
  groups: readonly GallerySidebarGroup<Id>[];
  descriptions: Readonly<Record<Id, string>>;
  selected: Id | null;
  compact?: boolean;
  onSelect(id: Id | null): void;
}

/** Searchable component navigation kept independent from router ownership. */
export function GallerySidebar<Id extends string>(
  props: GallerySidebarProps<Id>,
) {
  const [query, setQuery] = createSignal("");
  const filtered = createMemo(() =>
    filterSidebarGroups(
      props.groups,
      query(),
      (item) => `${item.name} ${props.descriptions[item.id] ?? ""}`,
    ),
  );
  const visibleCount = () =>
    filtered().reduce((total, group) => total + group.items.length, 0);
  const totalCount = () =>
    props.groups.reduce((total, group) => total + group.items.length, 0);

  return (
    <Sidebar
      aria-label="Component navigation"
      class={`h-full flex-none flex flex-col border-r border-subtle bg-surface-muted ${props.compact ? "w-48" : "w-56"}`}
    >
      <SidebarHeader class="h-14 px-4 flex items-center gap-3">
        <View class="w-8 h-8 flex items-center justify-center rounded-md bg-accent shadow-sm">
          <Text class="text-sm font-bold text-white">W</Text>
        </View>
        <View class="min-w-0 flex flex-col">
          <Text class="text-sm font-semibold text-primary">Wabou</Text>
          <Text class="truncate text-xs text-muted">Components & platform</Text>
        </View>
      </SidebarHeader>
      <SidebarSearch
        aria-label="Search components"
        value={query()}
        placeholder="Search components"
        clearLabel="Clear component search"
        onValueChange={setQuery}
      />
      <SidebarContent>
        <Show when={!query()}>
          <SidebarMenuButton
            aria-label="Overview"
            selected={props.selected === null}
            class="mb-3"
            onClick={() => props.onSelect(null)}
          >
            Overview
          </SidebarMenuButton>
        </Show>
        <ForValue each={filtered()}>
          {(group) => (
            <SidebarGroup aria-label={group.label}>
              <SidebarGroupLabel>{group.label}</SidebarGroupLabel>
              <ForValue each={group.items}>
                {(item) => (
                  <SidebarMenuButton
                    aria-label={item.name}
                    selected={props.selected === item.id}
                    onClick={() => props.onSelect(item.id)}
                  >
                    {item.name}
                  </SidebarMenuButton>
                )}
              </ForValue>
            </SidebarGroup>
          )}
        </ForValue>
        <Show when={visibleCount() === 0}>
          <SidebarEmpty
            title="No components found"
            description="Try a different search."
          />
        </Show>
      </SidebarContent>
      <SidebarFooter class="p-3">
        <Badge variant="outline">
          {query()
            ? `${visibleCount()} of ${totalCount()} showcases`
            : `${totalCount()} showcases`}
        </Badge>
      </SidebarFooter>
    </Sidebar>
  );
}
