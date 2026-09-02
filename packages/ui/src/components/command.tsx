import { mergeClasses } from "@wabou/core/style";
import {
  type Accessor,
  createEffect,
  createMemo,
  createSignal,
  For as ForValue,
  type JSX,
  Match,
  Switch,
} from "solid-js";
import { match } from "ts-pattern";
import { Text, View } from "../primitives";
import { Button } from "./button";
import {
  type CommandStateItem,
  filterCommandItems,
  reconcileCommandHighlight,
} from "./command-state";
import { Input } from "./input";
import { moveMenuHighlight } from "./menu-state";
import { pickerOptionClass } from "./select-semantics";

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
  loading?: boolean;
  loadingText?: string;
  error?: unknown;
  errorText?: string;
  retryLabel?: string;
  class?: string;
  listClass?: string;
  onQueryChange?: (query: string) => void;
  onAction?: (id: string) => void;
  onRetry?: () => void;
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
  loading?: boolean;
  loadingText?: string;
  error?: unknown;
  errorText?: string;
  retryLabel?: string;
  onRetry?: () => void;
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
      <Switch>
        <Match when={props.loading}>
          <Text role="status" class="px-3 py-4 text-sm text-muted text-center">
            {props.loadingText ?? "Loading results…"}
          </Text>
        </Match>
        <Match when={props.error !== undefined && props.error !== null}>
          <View
            role="alert"
            aria-label={props.errorText ?? "Could not load results"}
            class="px-3 py-3 flex flex-col items-center gap-2 text-center"
          >
            <Text class="text-sm font-medium text-danger-primary">
              {props.errorText ?? "Could not load results"}
            </Text>
            <Text class="max-w-full truncate text-xs text-danger-primary">
              {String(props.error)}
            </Text>
            {props.onRetry && (
              <Button
                size="sm"
                variant="outline"
                aria-label={props.retryLabel ?? "Try again"}
                onClick={props.onRetry}
              >
                {props.retryLabel ?? "Try again"}
              </Button>
            )}
          </View>
        </Match>
        <Match when={props.items.length === 0}>
          <Text role="status" class="px-3 py-4 text-sm text-muted text-center">
            {props.emptyText ?? "No results found."}
          </Text>
        </Match>
        <Match when={true}>
          <ForValue each={props.items} keyed={false}>
            {(item) => (
              <View
                id={item().id}
                role="option"
                aria-label={item().label}
                aria-selected={props.highlighted === item().id}
                aria-disabled={item().disabled}
                class={mergeClasses(
                  "min-h-8 px-2 py-1 flex flex-row items-center gap-2 rounded-md",
                  pickerOptionClass(
                    item().disabled ?? false,
                    props.highlighted === item().id,
                  ),
                  props.itemClass,
                )}
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
        </Match>
      </Switch>
    </View>
  );
}

export interface CommandListNavigationOptions {
  onAction?: (id: string) => void;
  onDismiss?: () => void;
}

/**
 * Shared command-list navigation for any focus owner, including native editors.
 * The caller forwards key events while the list remains a passive popup.
 */
export function createCommandListNavigation(
  items: Accessor<readonly CommandStateItem[]>,
  options: CommandListNavigationOptions = {},
) {
  const [highlighted, setHighlighted] = createSignal<string>();

  createEffect(
    () => ({ items: items(), highlighted: highlighted() }),
    ({ items: nextItems, highlighted: current }) => {
      setHighlighted(reconcileCommandHighlight(nextItems, current));
    },
  );

  const select = (id: string | undefined) => {
    const item = items().find((candidate) => candidate.id === id);
    if (!item || item.disabled) return false;
    options.onAction?.(item.id);
    return true;
  };
  const move = (direction: "first" | "last" | "next" | "previous") => {
    const next = moveMenuHighlight(items(), highlighted(), direction);
    if (next === undefined) return false;
    setHighlighted(next);
    return true;
  };
  const handleKeyDown = (event: { key: string; preventDefault(): void }) => {
    const handled = match(event.key)
      .with("ArrowDown", () => move("next"))
      .with("ArrowUp", () => move("previous"))
      .with("Home", () => move("first"))
      .with("End", () => move("last"))
      .with("Enter", () => select(highlighted()))
      .with("Escape", () => {
        options.onDismiss?.();
        return options.onDismiss !== undefined;
      })
      .otherwise(() => false);
    if (handled) event.preventDefault();
    return handled;
  };

  return { highlighted, setHighlighted, select, move, handleKeyDown };
}

/** Searchable command list whose filtering and keyboard behavior are host-independent. */
export function Command(props: CommandProps): JSX.Element {
  const [uncontrolledQuery, setUncontrolledQuery] = createSignal(
    props.defaultQuery ?? "",
  );
  const query = () => props.query ?? uncontrolledQuery();
  const filtered = createMemo(() => filterCommandItems(props.items, query()));

  const setQuery = (next: string) => {
    if (props.query === undefined) setUncontrolledQuery(next);
    props.onQueryChange?.(next);
  };
  const navigation = createCommandListNavigation(filtered, {
    onAction: (id) => {
      filtered()
        .find((item) => item.id === id)
        ?.onSelect?.();
      props.onAction?.(id);
    },
    onDismiss: props.onDismiss,
  });

  return (
    <View class={mergeClasses("min-w-0 flex flex-col gap-2", props.class)}>
      <Input
        aria-label={props["aria-label"]}
        value={query()}
        placeholder={props.placeholder ?? "Type a command"}
        ref={props.inputRef}
        onInput={(event) => setQuery(event.currentTarget.value)}
        onKeyDown={navigation.handleKeyDown}
      />
      <CommandList
        aria-label={`${props["aria-label"]} results`}
        items={filtered()}
        highlighted={navigation.highlighted()}
        emptyText={props.emptyText}
        loading={props.loading}
        loadingText={props.loadingText}
        error={props.error}
        errorText={props.errorText}
        retryLabel={props.retryLabel}
        onRetry={props.onRetry}
        class={props.listClass}
        onHighlightChange={navigation.setHighlighted}
        onAction={navigation.select}
      />
    </View>
  );
}

export { filterCommandItems, reconcileCommandHighlight } from "./command-state";

import type { Handle } from "@wabou/core/renderer";
