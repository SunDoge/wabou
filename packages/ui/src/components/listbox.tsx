import type { Handle } from "@wabou/core/renderer";
import { mergeClasses } from "@wabou/core/style";
import check from "lucide-static/icons/check.svg?raw";
import {
  createEffect,
  createSignal,
  For as ForValue,
  type JSX,
} from "solid-js";
import { match } from "ts-pattern";
import { Icon, ScrollArea, Text, View } from "../primitives";
import { moveMenuHighlight } from "./menu-state";
import { createControllableState } from "./state";
import { componentsDisabledItemClass } from "./theme";

export interface ListboxOption {
  value: string;
  label: string;
  /** Accessible identity when the visible label is not unique or sufficiently descriptive. */
  accessibilityLabel?: string;
  description?: string;
  disabled?: boolean;
}

export interface ListboxProps {
  options: readonly ListboxOption[];
  value?: string;
  defaultValue?: string;
  "aria-label": string;
  emptyText?: string;
  class?: string;
  listClass?: string;
  itemClass?: string;
  maxVisibleItems?: number;
  /** Fixed row height used for both layout and viewport calculation. */
  itemHeight?: number;
  /** Explicit scroll viewport height for inspector and split-pane layouts. */
  viewportHeight?: number;
  /** Fill the available flex height instead of deriving a fixed viewport. */
  fill?: boolean;
  /** Receives the focusable listbox handle for dialog and popover composition. */
  ref?: (node: Handle) => void;
  renderLeading?: (option: ListboxOption) => JSX.Element;
  renderTrailing?: (option: ListboxOption) => JSX.Element;
  onValueChange?: (value: string) => void;
  /** Invoked after pointer or keyboard activation of an enabled option. */
  onAction?: (value: string) => void;
  onDismiss?: () => void;
}

const optionItems = (options: readonly ListboxOption[]) =>
  options.map((option) => ({
    id: option.value,
    label: option.label,
    disabled: option.disabled,
  }));

/**
 * Focusable inline single-selection list.
 *
 * Unlike Select, Listbox owns no popup or trigger. It is suitable for dialogs,
 * inspectors and other surfaces where the choices are already visible.
 */
export function Listbox(props: ListboxProps): JSX.Element {
  const state = createControllableState<string | undefined>({
    value: () => props.value,
    defaultValue: props.defaultValue,
    onChange: (value) => value !== undefined && props.onValueChange?.(value),
  });
  const [highlighted, setHighlighted] = createSignal<string>();
  const items = () => optionItems(props.options);

  createEffect(
    () => ({
      options: props.options,
      selected: state.value(),
      highlighted: highlighted(),
    }),
    ({ options, selected, highlighted: current }) => {
      const candidates = optionItems(options);
      if (candidates.some((item) => item.id === current && !item.disabled)) {
        return;
      }
      const selectedOption = candidates.find(
        (item) => item.id === selected && !item.disabled,
      );
      setHighlighted(
        selectedOption?.id ?? moveMenuHighlight(candidates, undefined, "first"),
      );
    },
  );

  const move = (direction: "first" | "last" | "next" | "previous") => {
    const next = moveMenuHighlight(items(), highlighted(), direction);
    if (next === undefined) return false;
    setHighlighted(next);
    return true;
  };
  const select = (value: string | undefined) => {
    const option = props.options.find((candidate) => candidate.value === value);
    if (!option || option.disabled) return false;
    state.set(option.value);
    props.onAction?.(option.value);
    return true;
  };
  const handleKeyDown = (event: { key: string; preventDefault(): void }) => {
    const handled = match(event.key)
      .with("ArrowDown", () => move("next"))
      .with("ArrowUp", () => move("previous"))
      .with("Home", () => move("first"))
      .with("End", () => move("last"))
      .with("Enter", () => select(highlighted()))
      .with(" ", () => select(highlighted()))
      .with("Escape", () => {
        props.onDismiss?.();
        return props.onDismiss !== undefined;
      })
      .otherwise(() => false);
    if (handled) event.preventDefault();
  };
  const visibleItems = () =>
    Math.max(1, Math.min(props.options.length, props.maxVisibleItems ?? 8));
  const itemHeight = () => Math.max(24, props.itemHeight ?? 40);
  const viewportHeight = () =>
    Math.max(1, props.viewportHeight ?? visibleItems() * itemHeight());

  return (
    <ScrollArea
      class={mergeClasses(
        "w-full",
        props.fill ? "min-h-0 flex-1" : "flex-none",
        props.class,
      )}
      contentClass={mergeClasses("gap-1", props.listClass)}
      style={props.fill ? undefined : { height: `${viewportHeight()}px` }}
    >
      <View
        ref={props.ref}
        role="listbox"
        aria-label={props["aria-label"]}
        aria-activedescendant={highlighted()}
        focusOrder={0}
        class="select-none min-w-0 flex flex-col gap-1"
        onKeyDown={handleKeyDown}
      >
        {props.options.length === 0 ? (
          <Text role="status" class="px-3 py-4 text-sm text-muted text-center">
            {props.emptyText ?? "No options available."}
          </Text>
        ) : (
          <ForValue each={props.options} keyed={false}>
            {(option) => {
              const selected = () => state.value() === option().value;
              const active = () => highlighted() === option().value;
              return (
                <View
                  id={option().value}
                  role="option"
                  aria-label={option().accessibilityLabel ?? option().label}
                  aria-selected={selected()}
                  aria-disabled={option().disabled}
                  class={mergeClasses(
                    "w-full min-h-8 px-2 py-1 flex-none flex flex-row items-center gap-2 rounded-md",
                    active() && !option().disabled
                      ? "bg-control-hover text-primary"
                      : "bg-transparent text-secondary",
                    props.itemClass,
                    componentsDisabledItemClass(option().disabled ?? false),
                  )}
                  style={{
                    height: itemHeight(),
                  }}
                  onPointerMove={() =>
                    !option().disabled && setHighlighted(option().value)
                  }
                  onClick={() => select(option().value)}
                >
                  {props.renderLeading?.(option())}
                  <View class="min-w-0 flex-1 flex flex-col justify-center">
                    <Text class="min-w-0 truncate text-sm">
                      {option().label}
                    </Text>
                    {option().description && (
                      <Text class="min-w-0 truncate text-xs text-muted">
                        {option().description}
                      </Text>
                    )}
                  </View>
                  {props.renderTrailing?.(option())}
                  <View aria-hidden="true" class="w-4 h-4 flex-none">
                    {selected() && (
                      <Icon source={check} class="text-accent" size={16} />
                    )}
                  </View>
                </View>
              );
            }}
          </ForValue>
        )}
      </View>
    </ScrollArea>
  );
}
