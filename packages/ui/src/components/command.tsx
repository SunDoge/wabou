import { mergeClasses } from "@wabou/core/style";
import {
  createEffect,
  createMemo,
  createSignal,
  For as ForValue,
  type JSX,
} from "solid-js";
import { match } from "ts-pattern";
import { Text, View } from "../primitives";
import {
  type CommandStateItem,
  filterCommandItems,
  reconcileCommandHighlight,
} from "./command-state";
import { Input } from "./input";
import { moveMenuHighlight } from "./menu-state";

export interface CommandItem extends CommandStateItem {
  description?: string;
  /** Human-readable platform shortcut, such as `Ctrl K` or `⌘ K`. */
  shortcut?: string;
  onSelect?: () => void;
}

export interface CommandProps {
  items: readonly CommandItem[];
  "aria-label": string;
  query?: string;
  defaultQuery?: string;
  placeholder?: string;
  emptyText?: string;
  class?: string;
  listClass?: string;
  onQueryChange?: (query: string) => void;
  onAction?: (id: string) => void;
  onDismiss?: () => void;
  inputRef?: (node: Handle) => void;
}

export interface CommandListProps {
  items: readonly CommandItem[];
  "aria-label": string;
  highlighted?: string;
  emptyText?: string;
  class?: string;
  itemClass?: string;
  onHighlightChange?: (id: string) => void;
  onAction?: (id: string) => void;
  renderLeading?: (item: CommandItem) => JSX.Element;
}

/** Reusable command-result surface for search fields and inline completions. */
export function CommandList(props: CommandListProps): JSX.Element {
  return (
    <View
      role="listbox"
      aria-label={props["aria-label"]}
      aria-activedescendant={props.highlighted}
      class={mergeClasses("min-w-0 flex flex-col gap-1", props.class)}
    >
      {props.items.length === 0 ? (
        <Text role="status" class="px-3 py-4 text-sm text-muted text-center">
          {props.emptyText ?? "No results found."}
        </Text>
      ) : (
        <ForValue each={props.items} keyed={false}>
          {(item) => (
            <View
              id={item().id}
              role="option"
              aria-label={item().label}
              aria-selected={props.highlighted === item().id}
              aria-disabled={item().disabled}
              class={mergeClasses(
                "min-h-9 px-3 py-1.5 flex flex-row items-center gap-2 rounded-lg",
                props.highlighted === item().id
                  ? "bg-control-hover text-primary"
                  : "bg-transparent text-secondary",
                props.itemClass,
              )}
              style={{ opacity: item().disabled ? 0.45 : 1 }}
              onPointerMove={() =>
                !item().disabled && props.onHighlightChange?.(item().id)
              }
              onClick={() => !item().disabled && props.onAction?.(item().id)}
            >
              {props.renderLeading?.(item())}
              <View class="min-w-0 flex-1 flex flex-col justify-center">
                <Text class="min-w-0 truncate text-sm">{item().label}</Text>
                {item().description && (
                  <Text class="min-w-0 truncate text-xs text-muted">
                    {item().description}
                  </Text>
                )}
              </View>
              {item().shortcut && (
                <Text
                  aria-hidden="true"
                  class="flex-none rounded border border-subtle bg-surface px-1.5 py-0.5 text-xs text-muted"
                >
                  {item().shortcut}
                </Text>
              )}
            </View>
          )}
        </ForValue>
      )}
    </View>
  );
}

/** Searchable command list whose filtering and keyboard behavior are host-independent. */
export function Command(props: CommandProps): JSX.Element {
  const [uncontrolledQuery, setUncontrolledQuery] = createSignal(
    props.defaultQuery ?? "",
  );
  const [highlighted, setHighlighted] = createSignal<string>();
  const query = () => props.query ?? uncontrolledQuery();
  const filtered = createMemo(() => filterCommandItems(props.items, query()));

  createEffect(
    () => ({ items: filtered(), highlighted: highlighted() }),
    ({ items, highlighted: current }) => {
      setHighlighted(reconcileCommandHighlight(items, current));
    },
  );

  const setQuery = (next: string) => {
    if (props.query === undefined) setUncontrolledQuery(next);
    props.onQueryChange?.(next);
  };
  const select = (id: string | undefined) => {
    const item = filtered().find((candidate) => candidate.id === id);
    if (!item || item.disabled) return false;
    item.onSelect?.();
    props.onAction?.(item.id);
    return true;
  };
  const move = (direction: "first" | "last" | "next" | "previous") => {
    const next = moveMenuHighlight(filtered(), highlighted(), direction);
    if (next === undefined) return false;
    setHighlighted(next);
    return true;
  };
  const onKeyDown = (event: { key: string; preventDefault(): void }) => {
    const handled = match(event.key)
      .with("ArrowDown", () => move("next"))
      .with("ArrowUp", () => move("previous"))
      .with("Home", () => move("first"))
      .with("End", () => move("last"))
      .with("Enter", () => select(highlighted()))
      .with("Escape", () => {
        props.onDismiss?.();
        return props.onDismiss !== undefined;
      })
      .otherwise(() => false);
    if (handled) event.preventDefault();
  };

  return (
    <View class={mergeClasses("min-w-0 flex flex-col gap-2", props.class)}>
      <Input
        aria-label={props["aria-label"]}
        value={query()}
        placeholder={props.placeholder ?? "Type a command"}
        ref={props.inputRef}
        onInput={(event) => setQuery(event.currentTarget.value)}
        onKeyDown={onKeyDown}
      />
      <CommandList
        aria-label={`${props["aria-label"]} results`}
        items={filtered()}
        highlighted={highlighted()}
        emptyText={props.emptyText}
        class={props.listClass}
        onHighlightChange={setHighlighted}
        onAction={select}
      />
    </View>
  );
}

export { filterCommandItems, reconcileCommandHighlight } from "./command-state";

import type { Handle } from "@wabou/core/renderer";
