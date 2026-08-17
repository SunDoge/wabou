import { Accessor } from "solid-js";
//#region src/collection.d.ts
interface CollectionItem {
  id: string;
  disabled?: boolean;
  textValue?: string;
}
interface Collection<T extends CollectionItem> {
  items(): readonly T[];
  find(id: string): T | undefined;
  indexOf(id: string): number;
  first(): T | undefined;
  last(): T | undefined;
  next(id: string | undefined, loop?: boolean): T | undefined;
  previous(id: string | undefined, loop?: boolean): T | undefined;
}
declare function createCollection<T extends CollectionItem>(source: () => readonly T[]): Collection<T>;
//#endregion
//#region src/machine.d.ts
interface UpdateResult<State, Command = never> {
  state: State;
  commands: readonly Command[];
}
type Update<State, Event, Command = never> = (state: State, event: Event) => UpdateResult<State, Command>;
interface MachineOptions<State, Event, Command> {
  initialState: State;
  update: Update<State, Event, Command>;
  execute?: (command: Command, send: (event: Event) => void) => void;
  onTransition?: (result: UpdateResult<State, Command>, event: Event) => void;
}
interface Machine<State, Event> {
  state: Accessor<State>;
  send(event: Event): boolean;
}
/** Solid adapter for an Elm-style pure update function and explicit commands. */
declare function createMachine<State, Event, Command = never>(options: MachineOptions<State, Event, Command>): Machine<State, Event>;
declare function unchanged<State, Command = never>(state: State): UpdateResult<State, Command>;
//#endregion
//#region src/disclosure.d.ts
type DisclosureEvent = {
  type: "OPEN";
} | {
  type: "CLOSE";
} | {
  type: "TOGGLE";
} | {
  type: "DISABLED";
  disabled: boolean;
};
interface DisclosureState {
  open: boolean;
  disabled: boolean;
}
declare function updateDisclosure(state: DisclosureState, event: DisclosureEvent): UpdateResult<DisclosureState>;
interface DisclosureOptions {
  open?: () => boolean | undefined;
  defaultOpen?: boolean;
  disabled?: () => boolean;
  onOpenChange?: (open: boolean) => void;
}
declare function createDisclosure(options?: DisclosureOptions): {
  open: import("solid-js").Accessor<boolean>;
  disabled: () => boolean;
  openDisclosure: () => boolean;
  close: () => boolean;
  toggle: () => boolean;
};
//#endregion
//#region src/roving-focus.d.ts
interface FocusTarget {
  focus(): void;
}
interface RovingFocusItem {
  id: string;
  target: FocusTarget;
  disabled?: () => boolean;
}
interface RovingFocusOptions {
  orientation?: () => "horizontal" | "vertical";
  loop?: boolean;
  onMove?: (id: string) => void;
}
declare function createRovingFocus(options?: RovingFocusOptions): {
  register(item: RovingFocusItem): () => void;
  move(current: string, key: string): boolean;
};
//#endregion
//#region src/selection.d.ts
type SelectionMode = "single" | "multiple";
type Selection = string | readonly string[] | undefined;
declare function toggleSelection(current: Selection, item: string, mode: SelectionMode, allowEmpty?: boolean): Selection;
declare function isSelected(selection: Selection, item: string): boolean;
//#endregion
//#region src/select.d.ts
interface SelectItem extends CollectionItem {}
interface SelectState {
  open: boolean;
  value: string | undefined;
  highlighted: string | undefined;
}
type SelectEvent = {
  type: "OPEN";
} | {
  type: "CLOSE";
} | {
  type: "TOGGLE";
} | {
  type: "ARROW_DOWN";
} | {
  type: "ARROW_UP";
} | {
  type: "HOME";
} | {
  type: "END";
} | {
  type: "HIGHLIGHT";
  id: string;
} | {
  type: "SELECT";
  id?: string;
} | {
  type: "TYPEAHEAD";
  id: string;
};
type SelectCommand = {
  type: "FOCUS_TRIGGER";
} | {
  type: "FOCUS_CONTENT";
} | {
  type: "SCROLL_TO_ITEM";
  id: string;
};
interface SelectUpdateOptions<T extends SelectItem> {
  items: readonly T[];
  loop?: boolean;
  closeOnSelect?: boolean;
}
declare function updateSelect<T extends SelectItem>(state: SelectState, event: SelectEvent, options: SelectUpdateOptions<T>): {
  state: SelectState;
  commands: readonly SelectCommand[];
};
interface SelectInteractionOptions<T extends SelectItem> {
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
declare function createSelectInteraction<T extends SelectItem>(options: SelectInteractionOptions<T>): {
  state: () => SelectState;
  open: import("solid-js").Accessor<boolean>;
  value: import("solid-js").Accessor<string | undefined>;
  highlighted: import("solid-js").SourceAccessor<string | undefined>;
  send: (event: SelectEvent) => boolean;
  typeahead(key: string): boolean;
};
//#endregion
//#region src/state.d.ts
interface ControllableStateOptions<T> {
  value: () => T | undefined;
  defaultValue: T;
  disabled?: () => boolean;
  onChange?: (value: T) => void;
}
interface ControllableState<T> {
  value: Accessor<T>;
  set(value: T): boolean;
}
declare function createControllableState<T>(options: ControllableStateOptions<T>): ControllableState<T>;
//#endregion
//#region src/typeahead.d.ts
interface TypeaheadOptions {
  timeout?: number;
  locale?: string;
}
interface Typeahead<T extends CollectionItem> {
  search(items: readonly T[], key: string, activeId?: string): T | undefined;
  reset(): void;
}
declare function createTypeahead<T extends CollectionItem>(options?: TypeaheadOptions): Typeahead<T>;
//#endregion
export { Collection, CollectionItem, ControllableState, ControllableStateOptions, DisclosureEvent, DisclosureOptions, DisclosureState, FocusTarget, Machine, MachineOptions, RovingFocusItem, RovingFocusOptions, SelectCommand, SelectEvent, SelectInteractionOptions, SelectItem, SelectState, SelectUpdateOptions, Selection, SelectionMode, Typeahead, TypeaheadOptions, Update, UpdateResult, createCollection, createControllableState, createDisclosure, createMachine, createRovingFocus, createSelectInteraction, createTypeahead, isSelected, toggleSelection, unchanged, updateDisclosure, updateSelect };
//# sourceMappingURL=index.d.mts.map