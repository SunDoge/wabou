import {
  type ButtonState,
  Button as HeadlessButton,
  Center,
  Text,
  View,
} from "@wabou/primitives";
import {
  createControllableState,
  createRovingFocus,
} from "@wabou/interactions";
import type { Handle } from "@wabou/solid-renderer";
import {
  createComponent,
  createContext,
  type JSX,
  onCleanup,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";

const join = (...values: Array<string | undefined | false>) =>
  values.filter(Boolean).join(" ");

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
      .with({ indeterminate: true }, () => "−")
      .with({ checked: true }, () => "✓")
      .otherwise(() => "");

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
          buttonState.focused && "border-focus",
          props.class,
        )
      }
      style={(buttonState) => ({
        opacity: buttonState.disabled ? 0.45 : 1,
      })}
      onClick={toggle}
    >
      <Center
        class={join(
          SELECTION_INDICATOR_CLASS,
          "rounded text-xs font-bold",
          boxColors(),
        )}
      >
        <Text class="text-xs font-bold text-on-accent">{indicator()}</Text>
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
  return createComponent(RadioContext.Provider, {
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
          buttonState.focused && "border-focus",
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
      .with("sm", () => "h-8 min-w-8 px-2 text-xs")
      .with("default", () => "h-9 min-w-9 px-2.5 text-sm")
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
          state.focused && "border-focus",
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
