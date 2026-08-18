import type { Handle } from "@wabou/core/renderer";
import {
  type ButtonState,
  Center,
  Button as HeadlessButton,
  Icon,
  Text,
  View,
} from "@wabou/primitives";
import {
  createControllableState,
  createRovingFocus,
} from "@wabou/primitives/interactions";
import check from "lucide-static/icons/check.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
import {
  createComponent,
  createContext,
  type JSX,
  onCleanup,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import { join } from "./class-names";

const SELECTION_INDICATOR_CLASS = "w-5 h-5 flex-none border";

export interface CheckboxProps {
  checked?: boolean;
  defaultChecked?: boolean;
  indeterminate?: boolean;
  disabled?: boolean;
  label?: string;
  "aria-label"?: string;
  class?: string;
  onCheckedChange?: (checked: boolean) => void;
}

export function Checkbox(props: CheckboxProps): JSX.Element {
  const state = createControllableState({
    value: () => props.checked,
    defaultValue: props.defaultChecked ?? false,
    disabled: () => props.disabled ?? false,
    onChange: props.onCheckedChange,
  });
  const checked = state.value;
  const toggle = () => {
    state.set(!checked());
  };
  const boxColors = () =>
    match({ checked: checked(), indeterminate: !!props.indeterminate })
      .with({ checked: true }, () => "bg-accent border-accent text-on-accent")
      .with(
        { indeterminate: true },
        () => "bg-accent border-accent text-on-accent",
      )
      .otherwise(() => "bg-input border-strong text-primary");
  const ariaChecked = () =>
    match({ checked: checked(), indeterminate: !!props.indeterminate })
      .with({ indeterminate: true }, () => "mixed" as const)
      .otherwise(({ checked }) => checked);
  const indicator = () =>
    match({ checked: checked(), indeterminate: !!props.indeterminate })
      .with({ indeterminate: true }, () => minus)
      .with({ checked: true }, () => check)
      .otherwise(() => undefined);

  return (
    <HeadlessButton
      unstyled
      role="checkbox"
      disabled={props.disabled}
      aria-label={props["aria-label"] ?? props.label}
      aria-checked={ariaChecked()}
      selected={checked()}
      class={(buttonState) =>
        join(
          "min-h-7 px-1 items-center gap-2 rounded-md border border-transparent",
          buttonState.hovered && "bg-control-hover",
          buttonState.focusVisible && "border-focus",
          props.class,
        )
      }
      style={(buttonState) => ({
        opacity: buttonState.disabled ? 0.45 : 1,
      })}
      onClick={toggle}
    >
      <Center
        aria-hidden="true"
        class={join(
          SELECTION_INDICATOR_CLASS,
          "rounded text-xs font-bold",
          boxColors(),
        )}
      >
        {indicator() && (
          <Icon
            source={indicator() as string}
            size={14}
            class="text-on-accent"
          />
        )}
      </Center>
      {props.label && <Text class="text-sm text-secondary">{props.label}</Text>}
    </HeadlessButton>
  );
}

interface RadioContextValue {
  value: () => string | undefined;
  select(value: string): void;
  disabled: () => boolean;
  register(value: string, node: Handle, disabled: () => boolean): () => void;
  move(value: string, key: string): boolean;
}

const RadioContext = createContext<RadioContextValue>();

export interface RadioGroupProps {
  value?: string;
  defaultValue?: string;
  disabled?: boolean;
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
}

export function RadioGroup(props: RadioGroupProps): JSX.Element {
  const state = createControllableState<string | undefined>({
    value: () => props.value,
    defaultValue: props.defaultValue,
    disabled: () => props.disabled ?? false,
    onChange: (value) => value !== undefined && props.onValueChange?.(value),
  });
  const value = state.value;
  const select = (next: string) => {
    state.set(next);
  };
  const roving = createRovingFocus({
    orientation: () => "vertical",
    onMove: select,
  });
  return createComponent(RadioContext, {
    value: {
      value,
      select,
      disabled: () => props.disabled ?? false,
      register: (id, target, disabled) =>
        roving.register({ id, target, disabled }),
      move: roving.move,
    },
    get children() {
      return (
        <View
          role="radiogroup"
          aria-label={props["aria-label"]}
          class={join("flex flex-col gap-3", props.class)}
        >
          {props.children}
        </View>
      );
    },
  });
}

export interface RadioGroupItemProps {
  value: string;
  label?: string;
  disabled?: boolean;
  class?: string;
}

export function RadioGroupItem(props: RadioGroupItemProps): JSX.Element {
  const group = useContext(RadioContext);
  if (!group) throw new Error("RadioGroupItem must be used inside RadioGroup");
  const checked = () => group.value() === props.value;
  const disabled = () => group.disabled() || (props.disabled ?? false);
  let unregister: (() => void) | undefined;
  onCleanup(() => unregister?.());
  return (
    <HeadlessButton
      unstyled
      role="radio"
      disabled={disabled()}
      selected={checked()}
      aria-label={props.label}
      aria-checked={checked()}
      ref={(node) => {
        unregister?.();
        unregister = group.register(props.value, node, disabled);
      }}
      class={(buttonState) =>
        join(
          "min-h-7 px-1 items-center gap-2 rounded-md border border-transparent",
          buttonState.hovered && "bg-control-hover",
          buttonState.focusVisible && "border-focus",
          props.class,
        )
      }
      style={(buttonState) => ({
        opacity: buttonState.disabled ? 0.45 : 1,
      })}
      onClick={() => group.select(props.value)}
      onKeyDown={(event) => {
        if (group.move(props.value, event.key)) event.preventDefault();
      }}
    >
      <Center
        aria-hidden="true"
        class={join(
          SELECTION_INDICATOR_CLASS,
          "rounded-full bg-input",
          match(checked())
            .with(true, () => "border-accent")
            .with(false, () => "border-strong")
            .exhaustive(),
        )}
      >
        {checked() && <View class="w-2.5 h-2.5 rounded-full bg-accent" />}
      </Center>
      {props.label && <Text class="text-sm text-secondary">{props.label}</Text>}
    </HeadlessButton>
  );
}

export interface ToggleProps {
  pressed?: boolean;
  defaultPressed?: boolean;
  disabled?: boolean;
  variant?: "default" | "outline";
  size?: "sm" | "default" | "lg";
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
  onPressedChange?: (pressed: boolean) => void;
}

export function Toggle(props: ToggleProps): JSX.Element {
  const state = createControllableState({
    value: () => props.pressed,
    defaultValue: props.defaultPressed ?? false,
    disabled: () => props.disabled ?? false,
    onChange: props.onPressedChange,
  });
  const pressed = state.value;
  const toggle = () => {
    state.set(!pressed());
  };
  const size = () =>
    match(props.size ?? "default")
      .with("sm", () => "h-6 min-w-6 px-2 text-xs")
      .with("default", () => "h-8 min-w-8 px-2.5 text-sm")
      .with("lg", () => "h-10 min-w-10 px-3 text-sm")
      .exhaustive();
  const colors = (state: ButtonState) =>
    match({ selected: pressed(), hovered: state.hovered })
      .with({ selected: true }, () => "bg-selected border-accent text-primary")
      .with({ hovered: true }, () => "bg-control-hover text-primary")
      .otherwise(() => "bg-transparent text-secondary");
  return (
    <HeadlessButton
      unstyled
      disabled={props.disabled}
      selected={pressed()}
      aria-label={props["aria-label"]}
      aria-pressed={pressed()}
      class={(state) =>
        join(
          "items-center justify-center rounded-md border font-medium",
          size(),
          colors(state),
          match(props.variant ?? "default")
            .with("outline", () => "border-strong")
            .with("default", () => "border-transparent")
            .exhaustive(),
          state.focusVisible && "border-focus",
          props.class,
        )
      }
      style={(state) => ({ opacity: state.disabled ? 0.45 : 1 })}
      onClick={toggle}
    >
      {props.children}
    </HeadlessButton>
  );
}

interface ToggleGroupContextValue {
  value: () => string | undefined;
  disabled: () => boolean;
  select(value: string): void;
  register(value: string, node: Handle, disabled: () => boolean): () => void;
  move(value: string, key: string): boolean;
}

const ToggleGroupContext = createContext<ToggleGroupContextValue>();

export interface ToggleGroupProps {
  type: "single";
  value?: string;
  defaultValue?: string;
  disabled?: boolean;
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
}

/** Shadcn-style single-value toggle group with native roving focus. */
export function ToggleGroup(props: ToggleGroupProps): JSX.Element {
  const state = createControllableState<string | undefined>({
    value: () => props.value,
    defaultValue: props.defaultValue,
    disabled: () => props.disabled ?? false,
    onChange: (value) => value !== undefined && props.onValueChange?.(value),
  });
  const roving = createRovingFocus({
    orientation: () => "horizontal",
    onMove: (value) => state.set(value),
  });
  const context: ToggleGroupContextValue = {
    value: state.value,
    disabled: () => props.disabled ?? false,
    select: (value) => state.set(value),
    register: (value, node, disabled) =>
      roving.register({ id: value, target: node, disabled }),
    move: roving.move,
  };
  return createComponent(ToggleGroupContext, {
    value: context,
    get children() {
      return (
        <View
          role="group"
          aria-label={props["aria-label"]}
          class={join(
            "flex flex-row items-center gap-0.5 rounded-md bg-control p-0.5",
            props.class,
          )}
        >
          {props.children}
        </View>
      );
    },
  });
}

export interface ToggleGroupItemProps {
  value: string;
  disabled?: boolean;
  class?: string;
  children?: JSX.Element;
}

export function ToggleGroupItem(props: ToggleGroupItemProps): JSX.Element {
  const group = useContext(ToggleGroupContext);
  if (!group)
    throw new Error("ToggleGroupItem must be used inside ToggleGroup");
  const selected = () => group.value() === props.value;
  const disabled = () => group.disabled() || (props.disabled ?? false);
  let unregister: (() => void) | undefined;
  onCleanup(() => unregister?.());
  return (
    <HeadlessButton
      unstyled
      disabled={disabled()}
      selected={selected()}
      aria-pressed={selected()}
      ref={(node) => {
        unregister?.();
        unregister = group.register(props.value, node, disabled);
      }}
      class={(state) =>
        join(
          "h-7 flex-1 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium",
          selected()
            ? "bg-surface text-primary"
            : state.hovered
              ? "bg-control-hover text-primary"
              : "bg-transparent text-muted",
          state.focusVisible && "border-focus",
          props.class,
        )
      }
      style={(state) => ({ opacity: state.disabled ? 0.45 : 1 })}
      onClick={() => group.select(props.value)}
      onKeyDown={(event) => {
        if (group.move(props.value, event.key)) event.preventDefault();
      }}
    >
      {props.children}
    </HeadlessButton>
  );
}
