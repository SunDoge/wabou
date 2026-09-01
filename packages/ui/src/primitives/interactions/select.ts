import { createEffect, createSignal } from "solid-js";
import { match, P } from "ts-pattern";
import { createCollection, type CollectionItem } from "./collection";
import { createControllableState } from "./state";
import { createTypeahead } from "./typeahead";

export interface SelectItem extends CollectionItem {}
export interface SelectState {
  open: boolean;
  value: string | undefined;
  highlighted: string | undefined;
}
export type SelectEvent =
  | { type: "OPEN" }
  | { type: "CLOSE" }
  | { type: "TOGGLE" }
  | { type: "ARROW_DOWN" }
  | { type: "ARROW_UP" }
  | { type: "HOME" }
  | { type: "END" }
  | { type: "HIGHLIGHT"; id: string }
  | { type: "SELECT"; id?: string }
  | { type: "TYPEAHEAD"; id: string };
export type SelectCommand =
  | { type: "FOCUS_TRIGGER" }
  | { type: "FOCUS_CONTENT" }
  | { type: "SCROLL_TO_ITEM"; id: string };

export interface SelectUpdateOptions<T extends SelectItem> {
  items: readonly T[];
  loop?: boolean;
  closeOnSelect?: boolean;
}

export function updateSelect<T extends SelectItem>(
  state: SelectState,
  event: SelectEvent,
  options: SelectUpdateOptions<T>,
): { state: SelectState; commands: readonly SelectCommand[] } {
  const collection = createCollection(() => options.items);
  const initialHighlight = (fallback: string | undefined) => {
    const candidate = fallback ? collection.find(fallback) : undefined;
    return candidate && !candidate.disabled
      ? candidate.id
      : collection.first()?.id;
  };
  const openAt = (id: string | undefined) => ({
    state: { ...state, open: true, highlighted: id },
    commands: [
      { type: "FOCUS_CONTENT" as const },
      ...(id ? [{ type: "SCROLL_TO_ITEM" as const, id }] : []),
    ],
  });
  const move = (direction: "next" | "previous") => {
    const candidate =
      direction === "next"
        ? collection.next(state.highlighted, options.loop ?? true)
        : collection.previous(state.highlighted, options.loop ?? true);
    if (!candidate) return { state, commands: [] };
    return {
      state: { ...state, highlighted: candidate.id },
      commands: [{ type: "SCROLL_TO_ITEM" as const, id: candidate.id }],
    };
  };
  return match(event)
    .with({ type: "OPEN" }, () =>
      state.open
        ? { state, commands: [] }
        : openAt(initialHighlight(state.value)),
    )
    .with({ type: "CLOSE" }, () => ({
      state: { ...state, open: false, highlighted: undefined },
      commands: state.open ? [{ type: "FOCUS_TRIGGER" as const }] : [],
    }))
    .with({ type: "TOGGLE" }, () =>
      state.open
        ? {
            state: { ...state, open: false, highlighted: undefined },
            commands: [{ type: "FOCUS_TRIGGER" as const }],
          }
        : openAt(initialHighlight(state.value)),
    )
    .with({ type: "ARROW_DOWN" }, () =>
      state.open ? move("next") : openAt(initialHighlight(state.value)),
    )
    .with({ type: "ARROW_UP" }, () =>
      state.open
        ? move("previous")
        : openAt(state.value ?? collection.last()?.id),
    )
    .with({ type: "HOME" }, () => openAt(collection.first()?.id))
    .with({ type: "END" }, () => openAt(collection.last()?.id))
    .with(P.union({ type: "HIGHLIGHT" }, { type: "TYPEAHEAD" }), ({ id }) =>
      collection.find(id)?.disabled
        ? { state, commands: [] }
        : event.type === "TYPEAHEAD" && !state.open
          ? openAt(id)
          : {
              state: { ...state, highlighted: id },
              commands: [{ type: "SCROLL_TO_ITEM" as const, id }],
            },
    )
    .with({ type: "SELECT" }, ({ id }) => {
      const selected = id ?? state.highlighted;
      if (!selected || collection.find(selected)?.disabled)
        return { state, commands: [] };
      const close = options.closeOnSelect ?? true;
      return {
        state: {
          open: close ? false : state.open,
          value: selected,
          highlighted: close ? undefined : selected,
        },
        commands: close ? [{ type: "FOCUS_TRIGGER" as const }] : [],
      };
    })
    .exhaustive();
}

export interface SelectInteractionOptions<T extends SelectItem> {
  items: () => readonly T[];
  value?: () => string | undefined;
  defaultValue?: string;
  open?: () => boolean | undefined;
  defaultOpen?: boolean;
  disabled?: () => boolean;
  loop?: boolean;
  onValueChange?: (value: string) => void;
  onOpenChange?: (open: boolean) => void;
  execute?: (command: SelectCommand) => void;
}

export function createSelectInteraction<T extends SelectItem>(
  options: SelectInteractionOptions<T>,
) {
  const value = createControllableState({
    value: options.value ?? (() => undefined),
    defaultValue: options.defaultValue,
    disabled: options.disabled,
    onChange: (next) => next && options.onValueChange?.(next),
  });
  const open = createControllableState({
    value: options.open ?? (() => undefined),
    defaultValue: options.defaultOpen ?? false,
    disabled: options.disabled,
    onChange: options.onOpenChange,
  });
  const [highlighted, setHighlighted] = createSignal<string>();
  const typeahead = createTypeahead<T>();
  const state = (): SelectState => ({
    open: open.value(),
    value: value.value(),
    highlighted: highlighted(),
  });
  let observedOpen = false;
  let observedOpenInitialized = false;
  createEffect(open.value, (current) => {
    if (!observedOpenInitialized) {
      observedOpenInitialized = true;
      observedOpen = current;
      if (current) options.execute?.({ type: "FOCUS_CONTENT" });
      return;
    }
    if (current === observedOpen) return;
    observedOpen = current;
    options.execute?.({
      type: current ? "FOCUS_CONTENT" : "FOCUS_TRIGGER",
    });
  });
  const send = (event: SelectEvent) => {
    if (options.disabled?.()) return false;
    const controlledOpen = options.open?.();
    const result = updateSelect(state(), event, {
      items: options.items(),
      loop: options.loop,
    });
    const previous = state();
    open.set(result.state.open);
    if (result.state.value !== undefined) value.set(result.state.value);
    const openRequestRejected =
      controlledOpen !== undefined && controlledOpen !== result.state.open;
    if (!openRequestRejected) {
      setHighlighted(result.state.highlighted);
    }
    for (const command of result.commands) {
      if (command.type === "SCROLL_TO_ITEM" && result.state.open) {
        options.execute?.(command);
      }
    }
    return (
      previous.open !== result.state.open ||
      previous.value !== result.state.value ||
      previous.highlighted !== result.state.highlighted
    );
  };
  return {
    state,
    open: open.value,
    value: value.value,
    highlighted,
    send,
    typeahead(key: string) {
      const item = typeahead.search(options.items(), key, highlighted());
      return item ? send({ type: "TYPEAHEAD", id: item.id }) : false;
    },
  };
}
