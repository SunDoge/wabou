import {
  Badge,
  Button,
  Icon,
  Input,
  PrimitiveButton,
  ScrollArea,
  Text,
  View,
} from "@wabou/ui";
import x from "lucide-static/icons/x.svg?raw";
import { createMemo, createSignal, For, Show } from "solid-js";

export interface SidebarItem<Id extends string = string> {
  id: Id;
  name: string;
}

export interface SidebarGroup<Id extends string = string> {
  label: string;
  items: readonly SidebarItem<Id>[];
}

function normalizeSearch(value: string): string {
  return value.trim().toLocaleLowerCase();
}

export function filterSidebarGroups<Id extends string>(
  groups: readonly SidebarGroup<Id>[],
  descriptions: Readonly<Record<Id, string>>,
  query: string,
): SidebarGroup<Id>[] {
  const needle = normalizeSearch(query);
  if (!needle) return groups.map((group) => ({ ...group }));
  return groups
    .map((group) => ({
      ...group,
      items: group.items.filter((item) =>
        normalizeSearch(
          `${group.label} ${item.name} ${descriptions[item.id] ?? ""}`,
        ).includes(needle),
      ),
    }))
    .filter((group) => group.items.length > 0);
}

export interface GallerySidebarProps<Id extends string> {
  groups: readonly SidebarGroup<Id>[];
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
    filterSidebarGroups(props.groups, props.descriptions, query()),
  );
  const visibleCount = () =>
    filtered().reduce((total, group) => total + group.items.length, 0);
  const totalCount = () =>
    props.groups.reduce((total, group) => total + group.items.length, 0);

  return (
    <View
      class={`h-full flex-none flex flex-col border-r border-subtle bg-surface-muted ${props.compact ? "w-48" : "w-56"}`}
    >
      <View class="h-14 flex-none px-4 flex items-center gap-3 border-b border-subtle bg-surface">
        <View class="w-8 h-8 flex items-center justify-center rounded-md bg-accent shadow-sm">
          <Text class="text-sm font-bold text-white">W</Text>
        </View>
        <View class="min-w-0 flex flex-col">
          <Text class="text-sm font-semibold text-primary">Wabou</Text>
          <Text class="truncate text-xs text-muted">Components & platform</Text>
        </View>
      </View>
      <View class="flex-none p-2 flex items-center gap-1 border-b border-subtle bg-surface">
        <Input
          aria-label="Search components"
          value={query()}
          placeholder="Search components"
          class="min-w-0 flex-1"
          onInput={(event) => setQuery(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key !== "Escape" || !query()) return;
            event.preventDefault();
            setQuery("");
          }}
        />
        <Show when={query()}>
          <Button
            size="icon"
            variant="ghost"
            aria-label="Clear component search"
            onClick={() => setQuery("")}
          >
            <Icon source={x} aria-hidden="true" size={16} />
          </Button>
        </Show>
      </View>
      <ScrollArea class="flex-1" contentClass="px-2 py-3">
        <Show when={!query()}>
          <PrimitiveButton
            unstyled
            aria-label="Overview"
            selected={props.selected === null}
            class={(state) =>
              `w-full h-8 px-3 mb-3 justify-start rounded-md text-sm ${
                props.selected === null
                  ? "bg-selected text-primary"
                  : state.hovered
                    ? "bg-control-hover text-primary"
                    : "bg-transparent text-secondary"
              } ${state.focusVisible ? "border border-focus" : ""}`
            }
            onClick={() => props.onSelect(null)}
          >
            Overview
          </PrimitiveButton>
        </Show>
        <For each={filtered()}>
          {(group) => (
            <View class="flex-none flex flex-col gap-0.5 mb-4">
              <Text class="px-2 py-1 text-xs font-medium text-muted">
                {group.label}
              </Text>
              <For each={group.items}>
                {(item) => (
                  <PrimitiveButton
                    unstyled
                    aria-label={item.name}
                    selected={props.selected === item.id}
                    class={(state) =>
                      `w-full h-8 px-3 justify-start rounded-md text-sm ${
                        props.selected === item.id
                          ? "bg-selected text-primary"
                          : state.hovered
                            ? "bg-control-hover text-primary"
                            : "bg-transparent text-secondary"
                      } ${state.focusVisible ? "border border-focus" : ""}`
                    }
                    onClick={() => props.onSelect(item.id)}
                  >
                    {item.name}
                  </PrimitiveButton>
                )}
              </For>
            </View>
          )}
        </For>
        <Show when={visibleCount() === 0}>
          <View class="px-3 py-6 flex flex-col items-center gap-1">
            <Text role="status" class="text-sm text-secondary">
              No components found
            </Text>
            <Text class="text-xs text-muted">Try a different search.</Text>
          </View>
        </Show>
      </ScrollArea>
      <View class="flex-none p-3 border-t border-subtle bg-surface">
        <Badge variant="outline">
          {query()
            ? `${visibleCount()} of ${totalCount()} showcases`
            : `${totalCount()} showcases`}
        </Badge>
      </View>
    </View>
  );
}
