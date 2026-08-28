import { mergeClasses } from "@wabou/core/style";
import {
  createEffect,
  createMemo,
  createSignal,
  For,
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
      <View
        role="listbox"
        aria-label={`${props["aria-label"]} results`}
        aria-activedescendant={highlighted()}
        class={mergeClasses("min-w-0 flex flex-col gap-1", props.listClass)}
      >
        {filtered().length === 0 ? (
          <Text role="status" class="px-3 py-4 text-sm text-muted text-center">
            {props.emptyText ?? "No results found."}
          </Text>
        ) : (
          <For each={filtered()} keyed={false}>
            {(item) => (
              <View
                id={item().id}
                role="option"
                aria-label={item().label}
                aria-selected={highlighted() === item().id}
                aria-disabled={item().disabled}
                class={mergeClasses(
                  "min-h-9 px-3 py-1.5 flex flex-col justify-center rounded-lg",
                  highlighted() === item().id
                    ? "bg-control-hover text-primary"
                    : "bg-transparent text-secondary",
                )}
                style={{ opacity: item().disabled ? 0.45 : 1 }}
                onPointerMove={() =>
                  !item().disabled && setHighlighted(item().id)
                }
                onClick={() => select(item().id)}
              >
                <Text class="text-sm">{item().label}</Text>
                {item().description && (
                  <Text class="text-xs text-muted">{item().description}</Text>
                )}
              </View>
            )}
          </For>
        )}
      </View>
    </View>
  );
}

export { filterCommandItems, reconcileCommandHighlight } from "./command-state";

import type { Handle } from "@wabou/core/renderer";
