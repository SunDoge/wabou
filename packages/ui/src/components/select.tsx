import type { Handle } from "@wabou/core/renderer";
import type { Shadow } from "@wabou/core/style";
import { mergeClasses } from "@wabou/core/style";
import check from "lucide-static/icons/check.svg?raw";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import { createUniqueId, For as ForValue, type JSX } from "solid-js";
import { match } from "ts-pattern";
import {
  Button as HeadlessButton,
  Icon,
  Popover,
  ScrollArea,
  Text,
  View,
} from "../primitives";
import {
  createSelectInteraction,
  type SelectCommand,
} from "../primitives/interactions";
import type { PopupMotionProps } from "./popover";
import {
  type PickerTriggerVariant,
  pickerTriggerClass,
  selectControlsId,
} from "./select-semantics";
import {
  componentsControlSize,
  componentsElevation,
  useComponentsTheme,
} from "./theme";

// A medium native option is 32px tall with a 4px inter-item gap. Keeping the
// scroll pitch equal to rendered geometry avoids keyboard navigation drift.
const ITEM_HEIGHT = 36;
const VISIBLE_ITEMS = 6;

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps extends PopupMotionProps {
  options: readonly SelectOption[];
  value?: string;
  defaultValue?: string;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  /** Receives the native trigger handle for explicit label/focus composition. */
  ref?: (node: Handle) => void;
  placeholder?: string;
  "aria-label": string;
  class?: string;
  triggerVariant?: PickerTriggerVariant;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  onValueChange?: (value: string) => void;
  onOpenChange?: (open: boolean) => void;
}

/** Shadcn-inspired single Select backed by Wabou-native interaction state. */
export function Select(props: SelectProps): JSX.Element {
  const theme = useComponentsTheme();
  const id = createUniqueId();
  let trigger: Handle | undefined;
  let content: Handle | undefined;
  let viewport: Handle | undefined;
  let scrollTop = 0;
  const items = () =>
    props.options.map((option) => ({
      id: option.value,
      textValue: option.label,
      disabled: option.disabled,
    }));
  const execute = (command: SelectCommand) => {
    match(command)
      .with({ type: "FOCUS_TRIGGER" }, () =>
        requestAnimationFrame(() => trigger?.focus()),
      )
      .with({ type: "FOCUS_CONTENT" }, () =>
        requestAnimationFrame(() => content?.focus()),
      )
      .with({ type: "SCROLL_TO_ITEM" }, ({ id }) => {
        const index = props.options.findIndex((option) => option.value === id);
        if (index < 0) return;
        const firstVisible = Math.floor(scrollTop / ITEM_HEIGHT);
        const lastVisible = firstVisible + VISIBLE_ITEMS - 1;
        const nextTop =
          index < firstVisible
            ? index * ITEM_HEIGHT
            : index > lastVisible
              ? (index - VISIBLE_ITEMS + 1) * ITEM_HEIGHT
              : scrollTop;
        if (nextTop !== scrollTop) {
          scrollTop = nextTop;
          requestAnimationFrame(() => viewport?.scrollTo({ top: nextTop }));
        }
      })
      .exhaustive();
  };
  const interaction = createSelectInteraction({
    items,
    value: () => props.value,
    defaultValue: props.defaultValue,
    open: () => props.open,
    defaultOpen: props.defaultOpen,
    disabled: () => props.disabled ?? false,
    onValueChange: props.onValueChange,
    onOpenChange: props.onOpenChange,
    execute,
  });
  const selected = () =>
    props.options.find((option) => option.value === interaction.value());
  const handleKeyDown = (event: { key: string; preventDefault(): void }) => {
    const handled = match(event.key)
      .with("ArrowDown", () => interaction.send({ type: "ARROW_DOWN" }))
      .with("ArrowUp", () => interaction.send({ type: "ARROW_UP" }))
      .with("Home", () => interaction.send({ type: "HOME" }))
      .with("End", () => interaction.send({ type: "END" }))
      .with("Enter", () =>
        interaction.send({ type: interaction.open() ? "SELECT" : "OPEN" }),
      )
      .with(" ", () =>
        interaction.send({ type: interaction.open() ? "SELECT" : "OPEN" }),
      )
      .with("Escape", () => interaction.send({ type: "CLOSE" }))
      .otherwise((key) => interaction.typeahead(key));
    if (handled) event.preventDefault();
  };

  return (
    <Popover
      contentRole="presentation"
      popupRole="listbox"
      open={interaction.open()}
      onOpenChange={(open) => {
        interaction.send({ type: open ? "OPEN" : "CLOSE" });
      }}
      placement="bottom-start"
      openOnPointerDown
      contentClass={mergeClasses(
        "w-72 p-1 rounded-lg border border-subtle bg-surface",
        props.contentClass,
      )}
      contentShadows={
        props.contentShadows === undefined
          ? componentsElevation(theme(), "floating")
          : props.contentShadows
      }
      // Match native selects and Kobalte's headless default: opening is
      // immediate unless an application explicitly opts into popup motion.
      motion={props.motion ?? false}
      trigger={(popover) => (
        <HeadlessButton
          unstyled
          role="combobox"
          disabled={props.disabled}
          aria-label={props["aria-label"]}
          aria-haspopup="listbox"
          aria-expanded={interaction.open()}
          aria-controls={selectControlsId(`${id}-listbox`, interaction.open())}
          aria-valuetext={selected()?.label}
          ref={(node) => {
            trigger = node;
            props.ref?.(node);
            popover.ref(node);
          }}
          class={(state) =>
            mergeClasses(
              "w-72 overflow-hidden justify-between border",
              componentsControlSize("default"),
              pickerTriggerClass(props.triggerVariant ?? "default", state),
              props.class,
            )
          }
          style={(state) => ({ opacity: state.disabled ? 0.45 : 1 })}
          onClick={popover.onClick}
          onPointerDown={popover.onPointerDown}
          onPointerCancel={popover.onPointerCancel}
          onKeyDown={(event) => {
            popover.onKeyDown(event);
            handleKeyDown(event);
          }}
        >
          <Text
            class={mergeClasses(
              "min-w-0 flex-1 text-left truncate",
              selected() ? "text-primary" : "text-muted",
            )}
          >
            {selected()?.label ?? props.placeholder ?? "Select an option"}
          </Text>
          <Icon source={chevronDown} class="flex-none text-muted" size={16} />
        </HeadlessButton>
      )}
    >
      <ScrollArea
        ref={(node) => {
          viewport = node;
          scrollTop = 0;
        }}
        class="w-full flex-none"
        contentClass="gap-1"
        style={{
          height: `${Math.max(1, Math.min(props.options.length, VISIBLE_ITEMS)) * ITEM_HEIGHT - 4}px`,
        }}
        onScroll={(event) => {
          scrollTop = event.scrollY ?? scrollTop;
        }}
      >
        <View
          id={`${id}-listbox`}
          ref={(node) => (content = node)}
          role="listbox"
          aria-label={props["aria-label"]}
          aria-activedescendant={
            interaction.highlighted()
              ? `${id}-option-${interaction.highlighted()}`
              : undefined
          }
          focusOrder={0}
          class="select-none min-w-0 flex flex-col gap-1"
          onKeyDown={handleKeyDown}
        >
          <ForValue each={props.options} keyed={false}>
            {(option) => {
              const selected = () => interaction.value() === option().value;
              const highlighted = () =>
                interaction.highlighted() === option().value;
              return (
                <View
                  id={`${id}-option-${option().value}`}
                  role="option"
                  aria-selected={selected()}
                  aria-disabled={option().disabled}
                  class={mergeClasses(
                    "w-full h-8 flex-none px-2.5 flex items-center justify-between gap-2 rounded-md text-sm leading-normal",
                    highlighted()
                      ? "bg-control-hover text-primary"
                      : "bg-transparent text-secondary",
                  )}
                  style={{ opacity: option().disabled ? 0.45 : 1 }}
                  // A floating listbox can be positioned underneath a
                  // stationary pointer, and its leaf hit target can change
                  // between the option and its text. Keep hover selection in
                  // sync with pointer routing instead of relying on a single
                  // enter boundary event.
                  onPointerMove={() =>
                    interaction.send({ type: "HIGHLIGHT", id: option().value })
                  }
                  onClick={() =>
                    interaction.send({ type: "SELECT", id: option().value })
                  }
                >
                  <Text class="min-w-0 flex-1 text-sm truncate">
                    {option().label}
                  </Text>
                  <View aria-hidden="true" class="w-4 h-4 flex-none">
                    {selected() && (
                      <Icon source={check} class="text-accent" size={16} />
                    )}
                  </View>
                </View>
              );
            }}
          </ForValue>
        </View>
      </ScrollArea>
    </Popover>
  );
}
