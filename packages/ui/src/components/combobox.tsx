import type { Handle } from "@wabou/core/renderer";
import type { Shadow } from "@wabou/core/style";
import { mergeClasses } from "@wabou/core/style";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import { createSignal, type JSX } from "solid-js";
import { Button as HeadlessButton, Icon, Popover, Text } from "../primitives";
import { Command, type CommandItem } from "./command";
import type { PopupMotionProps } from "./popover";
import {
  componentsControlSize,
  componentsElevation,
  useComponentsTheme,
} from "./theme";

export interface ComboboxOption extends CommandItem {
  value: string;
}

export interface ComboboxProps extends PopupMotionProps {
  options: readonly ComboboxOption[];
  "aria-label": string;
  value?: string;
  defaultValue?: string;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  placeholder?: string;
  searchPlaceholder?: string;
  emptyText?: string;
  class?: string;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  onValueChange?: (value: string) => void;
  onOpenChange?: (open: boolean) => void;
}

/** A searchable single-value picker built from Popover and Command. */
export function Combobox(props: ComboboxProps): JSX.Element {
  const theme = useComponentsTheme();
  const [uncontrolledValue, setUncontrolledValue] = createSignal(
    props.defaultValue,
  );
  const [uncontrolledOpen, setUncontrolledOpen] = createSignal(
    props.defaultOpen ?? false,
  );
  const [query, setQuery] = createSignal("");
  let trigger: Handle | undefined;
  let search: Handle | undefined;
  const value = () => props.value ?? uncontrolledValue();
  const open = () => props.open ?? uncontrolledOpen();
  const selected = () =>
    props.options.find((option) => option.value === value());
  const setOpen = (next: boolean) => {
    if (props.open === undefined) setUncontrolledOpen(next);
    props.onOpenChange?.(next);
    if (next) {
      setQuery("");
      requestAnimationFrame(() => search?.focus());
    } else requestAnimationFrame(() => trigger?.focus());
  };
  const select = (id: string) => {
    const option = props.options.find((candidate) => candidate.id === id);
    if (!option || option.disabled) return;
    if (props.value === undefined) setUncontrolledValue(option.value);
    option.onSelect?.();
    props.onValueChange?.(option.value);
    setOpen(false);
  };

  return (
    <Popover
      contentRole="presentation"
      popupRole="listbox"
      open={open()}
      onOpenChange={setOpen}
      placement="bottom-start"
      contentClass={mergeClasses(
        "w-72 p-2 rounded-xl border border-subtle bg-surface",
        props.contentClass,
      )}
      contentShadows={
        props.contentShadows === undefined
          ? componentsElevation(theme(), "floating")
          : props.contentShadows
      }
      motion={props.motion}
      trigger={(popover) => (
        <HeadlessButton
          unstyled
          role="combobox"
          disabled={props.disabled}
          aria-label={props["aria-label"]}
          aria-haspopup="listbox"
          aria-expanded={open()}
          ref={(node) => {
            trigger = node;
            popover.ref(node);
          }}
          class={(state) =>
            mergeClasses(
              "w-72 overflow-hidden justify-between border bg-input shadow-xs",
              componentsControlSize("default"),
              state.focused ? "border-focus" : "border-subtle",
              props.class,
            )
          }
          style={(state) => ({ opacity: state.disabled ? 0.45 : 1 })}
          onClick={popover.onClick}
          onKeyDown={popover.onKeyDown}
        >
          <Text
            class={mergeClasses(
              "min-w-0 flex-1 text-left truncate",
              selected() ? "text-primary" : "text-muted",
            )}
          >
            {selected()?.label ?? props.placeholder ?? "Select an option"}
          </Text>
          <Icon
            source={chevronDown}
            class="flex-none text-muted"
            size={16}
          />
        </HeadlessButton>
      )}
    >
      <Command
        aria-label={`${props["aria-label"]} search`}
        query={query()}
        onQueryChange={setQuery}
        placeholder={props.searchPlaceholder ?? "Search options"}
        emptyText={props.emptyText}
        items={props.options}
        onAction={select}
        onDismiss={() => setOpen(false)}
        inputRef={(node) => (search = node)}
      />
    </Popover>
  );
}
