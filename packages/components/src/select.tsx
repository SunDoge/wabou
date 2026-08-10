import {
  createSelectInteraction,
  type SelectCommand,
} from "@wabou/interactions";
import {
  Button as HeadlessButton,
  Popover,
  Text,
  View,
} from "@wabou/primitives";
import type { Handle } from "@wabou/solid-renderer";
import { createUniqueId, Index, type JSX } from "solid-js";
import { match } from "ts-pattern";

const join = (...values: Array<string | undefined | false>) =>
  values.filter(Boolean).join(" ");

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps {
  options: readonly SelectOption[];
  value?: string;
  defaultValue?: string;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  placeholder?: string;
  "aria-label": string;
  class?: string;
  contentClass?: string;
  onValueChange?: (value: string) => void;
  onOpenChange?: (open: boolean) => void;
}

/** Shadcn-inspired single Select backed by Wabou-native interaction state. */
export function Select(props: SelectProps): JSX.Element {
  const id = createUniqueId();
  let trigger: Handle | undefined;
  let content: Handle | undefined;
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
      // Native ScrollArea owns scrolling. Item visibility will become an
      // explicit host command when virtualized collections are introduced.
      .with({ type: "SCROLL_TO_ITEM" }, () => undefined)
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
      open={interaction.open()}
      onOpenChange={(open) => {
        interaction.send({ type: open ? "OPEN" : "CLOSE" });
      }}
      placement="bottom-start"
      contentClass={join(
        "w-72 p-1 rounded-lg border border-subtle bg-surface shadow-lg",
        props.contentClass,
      )}
      trigger={(popover) => (
        <HeadlessButton
          unstyled
          role="combobox"
          disabled={props.disabled}
          aria-label={props["aria-label"]}
          aria-haspopup="listbox"
          aria-expanded={interaction.open()}
          aria-controls={`${id}-listbox`}
          ref={(node) => {
            trigger = node;
            popover.ref(node);
          }}
          class={(state) =>
            join(
              "w-72 h-9 px-3 justify-between gap-3 rounded-md border bg-input text-sm",
              state.focused ? "border-focus" : "border-strong",
              props.class,
            )
          }
          style={(state) => ({ opacity: state.disabled ? 0.45 : 1 })}
          onClick={popover.onClick}
          onKeyDown={(event) => {
            popover.onKeyDown(event);
            handleKeyDown(event);
          }}
        >
          <Text
            class={join(
              "min-w-0 flex-1 text-left truncate",
              selected() ? "text-primary" : "text-muted",
            )}
          >
            {selected()?.label ?? props.placeholder ?? "Select an option"}
          </Text>
          <Text class="flex-none text-muted">⌄</Text>
        </HeadlessButton>
      )}
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
        tabIndex={0}
        class="min-w-0 flex flex-col gap-1"
        onKeyDown={handleKeyDown}
      >
        <Index each={props.options}>
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
                class={join(
                  "w-full h-9 flex-none px-3 flex items-center justify-between gap-3 rounded-md text-sm",
                  highlighted()
                    ? "bg-control-hover text-primary"
                    : "bg-transparent text-secondary",
                )}
                style={{ opacity: option().disabled ? 0.45 : 1 }}
                onPointerEnter={() =>
                  interaction.send({ type: "HIGHLIGHT", id: option().value })
                }
                onClick={() =>
                  interaction.send({ type: "SELECT", id: option().value })
                }
              >
                <Text class="min-w-0 flex-1 text-sm">{option().label}</Text>
                <Text class="w-4 flex-none text-sm text-accent">
                  {selected() ? "✓" : ""}
                </Text>
              </View>
            );
          }}
        </Index>
      </View>
    </Popover>
  );
}
