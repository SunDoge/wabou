import { type AnimationControls, animate } from "@wabou/animation";
import { createFps } from "@wabou/core/renderer";
import {
  type ButtonState,
  Button as HeadlessButton,
  type ButtonProps as HeadlessButtonProps,
  PasswordInput as PrimitivePasswordInput,
  type PasswordInputProps as PrimitivePasswordInputProps,
  TextArea as PrimitiveTextArea,
  type TextAreaProps as PrimitiveTextAreaProps,
  TextInput as PrimitiveTextInput,
  type TextInputProps as PrimitiveTextInputProps,
  Text,
  translate2d,
  View,
  type WabouStyle,
} from "@wabou/primitives";
import {
  createEffect,
  createSignal,
  type JSX,
  omit,
  onCleanup,
  untrack,
} from "solid-js";
import { match, P } from "ts-pattern";
import { join } from "./class-names";
import { normalizePercentage } from "./range";
import { componentsElevation, useComponentsTheme } from "./theme";

export * from "./avatar";
export * from "./config-editor";
export * from "./date-picker";
export * from "./dialog";
export * from "./disclosure";
export { Kbd, KbdGroup, Skeleton, Spinner } from "./display";
export * from "./forms";
export * from "./layout";
export * from "./select";
export {
  Checkbox,
  type CheckboxProps,
  RadioGroup,
  RadioGroupItem,
  type RadioGroupItemProps,
  type RadioGroupProps,
  Toggle,
  ToggleGroup,
  ToggleGroupItem,
  type ToggleGroupItemProps,
  type ToggleGroupProps,
  type ToggleProps,
} from "./selection";
export * from "./slider";
export {
  Tabs,
  TabsContent,
  TabsList,
  type TabsProps,
  TabsTrigger,
  type TabsTriggerProps,
} from "./tabs";
export {
  type ComponentsElevation,
  ComponentsProvider,
  type ComponentsProviderProps,
  type ComponentsTheme,
  componentsElevation,
  useComponentsTheme,
} from "./theme";
export * from "./title-bar";

export type ButtonVariant =
  | "default"
  | "secondary"
  | "outline"
  | "ghost"
  | "destructive";
export type ButtonSize = "sm" | "default" | "lg" | "icon";

export interface ButtonProps
  extends Omit<HeadlessButtonProps, "variant" | "tone"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  class?: string;
  style?: HeadlessButtonProps["style"];
}

function buttonColors(variant: ButtonVariant, state: ButtonState): string {
  const focus = state.focusVisible ? "border-focus" : "";
  const passiveBorder = (variant: ButtonVariant) =>
    match(variant)
      .with("outline", () => "border-strong")
      .with(
        P.union("default", "secondary", "ghost", "destructive"),
        () => "border-transparent",
      )
      .exhaustive();

  return match({ variant, pressed: state.pressed, hovered: state.hovered })
    .with({ variant: "default", pressed: true }, () =>
      join("bg-accent-pressed border-transparent text-on-accent", focus),
    )
    .with({ variant: "default", hovered: true }, () =>
      join("bg-accent-hover border-transparent text-on-accent", focus),
    )
    .with({ variant: "default" }, () =>
      join("bg-accent border-transparent text-on-accent", focus),
    )
    .with({ variant: "destructive", pressed: true }, () =>
      join("bg-danger-pressed border-transparent text-on-accent", focus),
    )
    .with({ variant: "destructive", hovered: true }, () =>
      join("bg-danger-hover border-transparent text-on-accent", focus),
    )
    .with({ variant: "destructive" }, () =>
      join("bg-danger border-transparent text-on-accent", focus),
    )
    .with({ variant: "secondary", pressed: true }, () =>
      join("bg-control-pressed border-transparent text-primary", focus),
    )
    .with({ variant: "secondary", hovered: true }, () =>
      join("bg-control-hover border-transparent text-primary", focus),
    )
    .with({ variant: "secondary" }, () =>
      join("bg-control border-transparent text-primary", focus),
    )
    .with({ pressed: true }, ({ variant }) =>
      join("bg-control-pressed text-secondary", passiveBorder(variant), focus),
    )
    .with({ hovered: true }, ({ variant }) =>
      join("bg-control-hover text-secondary", passiveBorder(variant), focus),
    )
    .with({ variant: P.union("outline", "ghost") }, ({ variant }) =>
      join("bg-transparent text-secondary", passiveBorder(variant), focus),
    )
    .exhaustive();
}

function buttonSize(size: ButtonSize): string {
  return match(size)
    .with("sm", () => "h-6 px-2 text-xs")
    .with("default", () => "h-8 px-3 text-sm")
    .with("lg", () => "h-10 px-4 text-base")
    .with("icon", () => "w-8 h-8 p-0 text-sm")
    .exhaustive();
}

export function Button(props: ButtonProps): JSX.Element {
  const local = props;
  const forwarded = omit(props, "variant", "size", "class", "style");
  const variant = () => local.variant ?? "default";
  const size = () => local.size ?? "default";
  return (
    <HeadlessButton
      {...forwarded}
      unstyled
      class={(state) =>
        join(
          "inline-flex flex-none whitespace-nowrap items-center justify-center rounded-md border font-medium",
          buttonColors(variant(), state),
          buttonSize(size()),
          local.class,
        )
      }
      style={(state) =>
        ({
          // Focus feedback must not change the content box. A wider focused
          // border makes labels jump by one pixel on every pointer click.
          "border-width": 1,
          opacity: state.disabled ? 0.45 : 1,
          ...(typeof local.style === "function"
            ? local.style(state)
            : local.style),
        }) as WabouStyle
      }
    />
  );
}

export interface BadgeProps {
  children?: JSX.Element;
  variant?: "default" | "secondary" | "outline" | "success" | "destructive";
  class?: string;
}

function badgeColors(variant: NonNullable<BadgeProps["variant"]>): string {
  return match(variant)
    .with("default", () => "bg-accent border-accent text-on-accent")
    .with("secondary", () => "bg-control border-subtle text-primary")
    .with("outline", () => "bg-transparent border-strong text-secondary")
    .with(
      "success",
      () => "bg-success-surface border-success-primary text-success-primary",
    )
    .with(
      "destructive",
      () => "bg-danger-surface border-danger text-danger-primary",
    )
    .exhaustive();
}

export function Badge(props: BadgeProps): JSX.Element {
  return (
    <Text
      class={join(
        "flex-none whitespace-nowrap px-2 py-0.5 rounded-md border text-xs font-medium",
        badgeColors(props.variant ?? "default"),
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export interface FpsProps {
  /** Controlled FPS value. When omitted, the component measures host frames. */
  value?: number;
  /** Text displayed after the value. Set to an empty string for value only. */
  label?: string;
  /** FPS at or above this value uses the success treatment. */
  goodAt?: number;
  /** FPS below this value uses the destructive treatment. */
  warningBelow?: number;
  class?: string;
}

/** Live host frame-rate indicator with sensible performance thresholds. */
export function Fps(props: FpsProps): JSX.Element {
  const measured =
    props.value === undefined ? createFps() : () => props.value ?? 0;
  const value = () => Math.max(0, Math.round(measured()));
  const variant = (): BadgeProps["variant"] =>
    match(value())
      .with(0, () => "outline" as const)
      .with(
        P.when((fps) => fps >= (props.goodAt ?? 55)),
        () => "success" as const,
      )
      .with(
        P.when((fps) => fps < (props.warningBelow ?? 30)),
        () => "destructive" as const,
      )
      .otherwise(() => "secondary");
  return (
    <Badge variant={variant()} class={join("font-mono", props.class)}>
      {value()}
      {props.label === "" ? "" : ` ${props.label ?? "fps"}`}
    </Badge>
  );
}

export function Card(props: {
  children?: JSX.Element;
  class?: string;
  shadows?: readonly import("@wabou/core/style").Shadow[] | null;
}): JSX.Element {
  const theme = useComponentsTheme();
  return (
    <View
      class={join(
        "flex flex-col overflow-hidden rounded-lg border",
        "border-subtle bg-surface",
        props.class,
      )}
      shadows={
        props.shadows === undefined
          ? componentsElevation(theme(), "raised")
          : props.shadows
      }
    >
      {props.children}
    </View>
  );
}
export function CardHeader(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <View class={join("flex flex-col gap-1 px-4 pt-4", props.class)}>
      {props.children}
    </View>
  );
}
export function CardTitle(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <Text class={join("text-base font-semibold", "text-primary", props.class)}>
      {props.children}
    </Text>
  );
}
export function CardDescription(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <Text
      class={join(
        "w-full min-w-0 whitespace-normal text-sm",
        "text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}
export function CardContent(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <View class={join("flex flex-col gap-3 p-4", props.class)}>
      {props.children}
    </View>
  );
}
export function CardFooter(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <View class={join("flex items-center gap-2 px-4 pb-4", props.class)}>
      {props.children}
    </View>
  );
}

export function Separator(props: {
  orientation?: "horizontal" | "vertical";
  class?: string;
}): JSX.Element {
  const dimensions = () =>
    match(props.orientation ?? "horizontal")
      .with("horizontal", () => "h-px w-full")
      .with("vertical", () => "w-px h-full")
      .exhaustive();
  return (
    <View
      aria-hidden="true"
      class={join("flex-none", "bg-subtle", dimensions(), props.class)}
    />
  );
}

export function Alert(props: {
  title: string;
  children?: JSX.Element;
  variant?: "default" | "destructive";
  class?: string;
}): JSX.Element {
  const colors = () =>
    match(props.variant ?? "default")
      .with("default", () => ({
        container: "border-subtle bg-surface",
        title: "text-primary",
        body: "text-secondary",
      }))
      .with("destructive", () => ({
        container: "border-danger bg-danger-surface",
        title: "text-danger-primary",
        body: "text-danger-primary",
      }))
      .exhaustive();
  return (
    <View
      role="alert"
      aria-label={props.title}
      class={join(
        "flex flex-col gap-1 rounded-lg border p-4 shadow-xs",
        colors().container,
        props.class,
      )}
    >
      <Text class={join("text-sm font-semibold", colors().title)}>
        {props.title}
      </Text>
      <Text
        class={join("w-full min-w-0 whitespace-normal text-sm", colors().body)}
      >
        {props.children}
      </Text>
    </View>
  );
}

export interface InputProps extends PrimitiveTextInputProps {
  class?: string;
}

/** A plain-text input. Secrets must use {@link PasswordInput}. */
export function Input(props: InputProps): JSX.Element {
  return (
    <PrimitiveTextInput
      {...props}
      class={join(
        "h-8 w-full px-3 rounded-md border text-sm shadow-xs",
        "border-subtle bg-input text-primary",
        props.disabled && "opacity-50",
        props.class,
      )}
    />
  );
}

export interface PasswordInputProps extends PrimitivePasswordInputProps {
  class?: string;
}

/** A native secret input whose value never crosses into JavaScript. */
export function PasswordInput(props: PasswordInputProps): JSX.Element {
  return (
    <PrimitivePasswordInput
      {...props}
      class={join(
        "h-8 w-full px-3 rounded-md border text-sm shadow-xs",
        "border-subtle bg-input text-primary",
        props.disabled && "opacity-50",
        props.class,
      )}
    />
  );
}

export interface TextAreaProps extends PrimitiveTextAreaProps {
  class?: string;
}

export function TextArea(props: TextAreaProps): JSX.Element {
  return (
    <PrimitiveTextArea
      {...props}
      class={join(
        "h-24 w-full px-3 py-2 rounded-md border text-sm shadow-xs",
        "border-subtle bg-input text-primary",
        props.disabled && "opacity-50",
        props.class,
      )}
    />
  );
}

export interface SwitchProps {
  checked?: boolean;
  defaultChecked?: boolean;
  disabled?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  label?: string;
  "aria-label"?: string;
}

function switchColors(checked: boolean, state: ButtonState): string {
  return match({ checked, pressed: state.pressed, hovered: state.hovered })
    .with({ checked: true, pressed: true }, () => "bg-accent-pressed")
    .with({ checked: true, hovered: true }, () => "bg-accent-hover")
    .with({ checked: true }, () => "bg-accent")
    .with({ checked: false, pressed: true }, () => "bg-control-pressed")
    .with({ checked: false, hovered: true }, () => "bg-control-hover")
    .with({ checked: false }, () => "bg-control")
    .exhaustive();
}

export function Switch(props: SwitchProps): JSX.Element {
  const [local, setLocal] = createSignal(props.defaultChecked ?? false);
  const checked = () => props.checked ?? local();
  const [thumbX, setThumbX] = createSignal(checked() ? 20 : 0);
  let movement: AnimationControls | undefined;
  createEffect(checked, (isChecked) => {
    const target = isChecked ? 20 : 0;
    const from = untrack(thumbX);
    if (from === target) return;
    movement?.stop();
    movement = animate(from, target, {
      duration: 0.18,
      ease: [0.22, 1, 0.36, 1],
      onUpdate: setThumbX,
    });
  });
  onCleanup(() => movement?.stop());
  const toggle = () => {
    if (props.disabled) return;
    const next = !checked();
    if (props.checked === undefined) setLocal(next);
    props.onCheckedChange?.(next);
  };
  return (
    <View class="flex items-center gap-3">
      <HeadlessButton
        unstyled
        role="switch"
        disabled={props.disabled}
        aria-label={props["aria-label"] ?? props.label}
        aria-checked={checked()}
        class={(state) =>
          join(
            "w-11 h-6 flex-none rounded-full p-0.5",
            switchColors(checked(), state),
            state.focused && "border border-focus",
          )
        }
        style={(state) => ({
          opacity: state.disabled ? 0.45 : 1,
        })}
        onClick={toggle}
      >
        <View
          aria-hidden="true"
          class="w-5 h-5 rounded-full bg-on-accent"
          transform={translate2d(thumbX(), 0)}
        />
      </HeadlessButton>
      {props.label && <Text class="text-sm text-secondary">{props.label}</Text>}
    </View>
  );
}

export function Progress(props: {
  value?: number;
  label?: string;
  class?: string;
}): JSX.Element {
  const value = () => normalizePercentage(props.value);
  return (
    <View
      role="progressbar"
      aria-label={props.label ?? "Progress"}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={value()}
      aria-valuetext={`${value()} percent`}
      class={join(
        "w-full h-2 overflow-hidden rounded-full",
        "bg-control",
        props.class,
      )}
    >
      <View
        aria-hidden="true"
        class="h-full bg-accent rounded-full"
        style={{ width: `${value()}%` }}
      />
    </View>
  );
}
