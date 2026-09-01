import type { Handle } from "@wabou/core/renderer";
import { mergeClasses } from "@wabou/core/style";
import check from "lucide-static/icons/check.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
import {
  createComponent,
  createContext,
  createSignal,
  type JSX,
  onCleanup,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import {
  type ButtonState,
  Center,
  Button as HeadlessButton,
  Icon,
  Text,
  View,
} from "../primitives";
import {
  createControllableState,
  createRovingFocus,
} from "../primitives/interactions";
import { componentsControlSize } from "./theme";

const SELECTION_INDICATOR_CLASS = "w-4 h-4 flex-none border";

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
        mergeClasses(
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
        class={mergeClasses(
          SELECTION_INDICATOR_CLASS,
          "rounded text-xs font-bold",
          boxColors(),
        )}
      >
        {indicator() && (
          <Icon
            source={indicator() as string}
            size={12}
            class="text-on-accent"
          />
        )}
      </Center>
      {props.label && <Text class="text-sm text-primary">{props.label}</Text>}
    </HeadlessButton>
  );
}

interface RadioContextValue {
  value: () => string | undefined;
  select(value: string): void;
  disabled: () => boolean;
  appearance: () => "radio" | "segment";
  register(value: string, node: Handle, disabled: () => boolean): () => void;
  move(value: string, key: string): boolean;
}

const RadioContext = createContext<RadioContextValue>();

export interface RadioGroupProps {
  value?: string;
  defaultValue?: string;
  disabled?: boolean;
  orientation?: "horizontal" | "vertical";
  appearance?: "radio" | "segment";
  loop?: boolean;
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
    orientation: () => props.orientation ?? "vertical",
    loop: props.loop,
    onMove: select,
  });
  return createComponent(RadioContext, {
    value: {
      value,
      select,
      disabled: () => props.disabled ?? false,
      appearance: () => props.appearance ?? "radio",
      register: (id, target, disabled) =>
        roving.register({ id, target, disabled }),
      move: roving.move,
    },
    get children() {
      return (
        <View
          role="radiogroup"
          aria-label={props["aria-label"]}
          class={mergeClasses(
            "flex",
            props.orientation === "horizontal" ? "flex-row" : "flex-col",
            props.appearance === "segment"
              ? "items-center gap-0.5 rounded-md border border-subtle bg-control p-0.5"
              : "gap-3",
            props.class,
          )}
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
        mergeClasses(
          group.appearance() === "segment"
            ? "h-8 min-w-0 flex-1 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium"
            : "min-h-7 px-1 items-center gap-2 rounded-md border border-transparent",
          group.appearance() === "segment" && checked()
            ? "bg-selected text-primary shadow-xs"
            : buttonState.hovered && "bg-control-hover",
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
      {group.appearance() === "radio" && (
        <Center
          aria-hidden="true"
          class={mergeClasses(
            SELECTION_INDICATOR_CLASS,
            "rounded-full bg-input",
            match(checked())
              .with(true, () => "border-accent")
              .with(false, () => "border-strong")
              .exhaustive(),
          )}
        >
          {checked() && <View class="w-2 h-2 rounded-full bg-accent" />}
        </Center>
      )}
      {props.label && (
        <Text
          class={mergeClasses(
            "text-sm",
            checked() ? "text-primary" : "text-secondary",
          )}
        >
          {props.label}
        </Text>
      )}
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
      .with("sm", () => `${componentsControlSize("sm")} min-w-7`)
      .with("default", () => `${componentsControlSize("default")} min-w-8`)
      .with("lg", () => `${componentsControlSize("lg")} min-w-10`)
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
        mergeClasses(
          "items-center justify-center border font-medium",
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
  selected(value: string): boolean;
  disabled: () => boolean;
  toggle(value: string): void;
  register(value: string, node: Handle, disabled: () => boolean): () => void;
  activate(value: string): void;
  isTabStop(value: string): boolean;
  move(value: string, key: string): boolean;
  variant: () => "default" | "outline";
  size: () => "sm" | "default" | "lg";
}

const ToggleGroupContext = createContext<ToggleGroupContextValue>();

interface ToggleGroupBaseProps {
  disabled?: boolean;
  "aria-label"?: string;
  variant?: "default" | "outline";
  size?: "sm" | "default" | "lg";
  spacing?: 0 | 1 | 2;
  loop?: boolean;
  class?: string;
  children?: JSX.Element;
}

export type ToggleGroupProps = ToggleGroupBaseProps &
  (
    | {
        type?: "single";
        value?: string;
        defaultValue?: string;
        onValueChange?: (value: string) => void;
      }
    | {
        type: "multiple";
        value?: readonly string[];
        defaultValue?: readonly string[];
        onValueChange?: (value: readonly string[]) => void;
      }
  );

export function nextToggleGroupValue(
  current: string | readonly string[],
  value: string,
  type: "single" | "multiple",
): string | readonly string[] {
  if (type === "single") return current === value ? "" : value;
  const values = Array.isArray(current) ? current : [];
  return values.includes(value)
    ? values.filter((candidate) => candidate !== value)
    : [...values, value];
}

/** Shadcn-style single-value toggle group with native roving focus. */
export function ToggleGroup(props: ToggleGroupProps): JSX.Element {
  const entries: Array<{ value: string; disabled: () => boolean }> = [];
  const [activeValue, setActiveValue] = createSignal<string | undefined>(
    undefined,
    { ownedWrite: true },
  );
  const [registryVersion, setRegistryVersion] = createSignal(0, {
    ownedWrite: true,
  });
  const type = () => props.type ?? "single";
  const state = createControllableState<string | readonly string[]>({
    value: () => props.value,
    defaultValue: props.defaultValue ?? (props.type === "multiple" ? [] : ""),
    disabled: () => props.disabled ?? false,
    onChange: (value) => {
      if (props.type === "multiple") {
        props.onValueChange?.(Array.isArray(value) ? value : []);
      } else {
        props.onValueChange?.(typeof value === "string" ? value : "");
      }
    },
  });
  const activateFromKeyboard = (value: string) => {
    setActiveValue(value);
    if (type() === "single" && state.value() !== value) state.set(value);
  };
  const roving = createRovingFocus({
    orientation: () => "horizontal",
    loop: props.loop,
    onMove: activateFromKeyboard,
  });
  const context: ToggleGroupContextValue = {
    selected(value) {
      const current = state.value();
      return Array.isArray(current)
        ? current.includes(value)
        : current === value;
    },
    disabled: () => props.disabled ?? false,
    toggle: (value) =>
      state.set(nextToggleGroupValue(state.value(), value, type())),
    register(value, node, disabled) {
      const entry = { value, disabled };
      entries.push(entry);
      const unregisterRoving = roving.register({
        id: value,
        target: node,
        disabled,
      });
      setRegistryVersion((version) => version + 1);
      return () => {
        unregisterRoving();
        const index = entries.indexOf(entry);
        if (index >= 0) entries.splice(index, 1);
        setRegistryVersion((version) => version + 1);
      };
    },
    activate: setActiveValue,
    isTabStop(value) {
      registryVersion();
      const enabled = entries.filter((entry) => !entry.disabled());
      const active = activeValue();
      const current = enabled.some((entry) => entry.value === active)
        ? active
        : (enabled.find((entry) => context.selected(entry.value))?.value ??
          enabled[0]?.value);
      return value === current;
    },
    move: roving.move,
    variant: () => props.variant ?? "default",
    size: () => props.size ?? "default",
  };
  return createComponent(ToggleGroupContext, {
    value: context,
    get children() {
      return (
        <View
          role="group"
          aria-label={props["aria-label"]}
          class={mergeClasses(
            "flex flex-row items-center rounded-md bg-transparent",
            match(props.spacing ?? 0)
              .with(0, () => "gap-0")
              .with(1, () => "gap-1")
              .with(2, () => "gap-2")
              .exhaustive(),
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
  variant?: "default" | "outline" | "accent";
  size?: "sm" | "default" | "lg";
  class?: string;
  children?: JSX.Element;
}

export function ToggleGroupItem(props: ToggleGroupItemProps): JSX.Element {
  const group = useContext(ToggleGroupContext);
  if (!group)
    throw new Error("ToggleGroupItem must be used inside ToggleGroup");
  const selected = () => group.selected(props.value);
  const disabled = () => group.disabled() || (props.disabled ?? false);
  let unregister: (() => void) | undefined;
  onCleanup(() => unregister?.());
  return (
    <HeadlessButton
      unstyled
      disabled={disabled()}
      selected={selected()}
      aria-pressed={selected()}
      focusOrder={group.isTabStop(props.value) ? 0 : -1}
      ref={(node) => {
        unregister?.();
        unregister = group.register(props.value, node, disabled);
      }}
      class={(state) =>
        mergeClasses(
          "h-7 flex-1 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium",
          match(props.size ?? group.size())
            .with("sm", () => componentsControlSize("sm"))
            .with("default", () => componentsControlSize("default"))
            .with("lg", () => componentsControlSize("lg"))
            .exhaustive(),
          match({
            selected: selected(),
            accent: props.variant === "accent",
            hovered: state.hovered,
          })
            .with(
              { selected: true, accent: true },
              () => "bg-accent text-on-accent",
            )
            .with({ selected: true }, () => "bg-selected text-primary")
            .with({ hovered: true }, () => "bg-control-hover text-primary")
            .otherwise(() => "bg-transparent text-muted"),
          (props.variant ?? group.variant()) === "outline" && "border-strong",
          state.focusVisible && "border-focus",
          props.class,
        )
      }
      style={(state) => ({ opacity: state.disabled ? 0.45 : 1 })}
      onFocus={() => group.activate(props.value)}
      onClick={() => group.toggle(props.value)}
      onKeyDown={(event) => {
        if (group.move(props.value, event.key)) event.preventDefault();
      }}
    >
      {props.children}
    </HeadlessButton>
  );
}
