import { createFps } from "@wabou/core/renderer";
import {
  createEffect,
  createSignal,
  type JSX,
  onCleanup,
  untrack,
} from "solid-js";
import { match, P } from "ts-pattern";
import { type AnimationControls, animate } from "../animation";
import {
  type ButtonState,
  Button as HeadlessButton,
  PasswordInput as PrimitivePasswordInput,
  type PasswordInputProps as PrimitivePasswordInputProps,
  TextArea as PrimitiveTextArea,
  type TextAreaProps as PrimitiveTextAreaProps,
  Text,
  type TextProps,
  translate2d,
  View,
  type ViewProps,
} from "../primitives";
import { join } from "./class-names";
import { normalizePercentage } from "./range";
import { componentsElevation, useComponentsTheme } from "./theme";

export * from "./alert-dialog";
export * from "./avatar";
export * from "./button";
export * from "./combobox";
export * from "./command";
export * from "./config-editor";
export * from "./context-menu";
export * from "./date-picker";
export * from "./dialog";
export * from "./directory-picker";
export * from "./disclosure";
export { Kbd, KbdGroup, Skeleton, Spinner } from "./display";
export * from "./dropdown-menu";
export * from "./forms";
export * from "./hover-card";
export * from "./input";
export * from "./layout";
export * from "./navigation";
export * from "./page";
export * from "./popover";
export * from "./resizable";
export * from "./search-field";
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
export * from "./sheet";
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
export * from "./toast";
export * from "./tooltip";
export * from "./tree-view";

export interface BadgeProps {
  children?: JSX.Element;
  variant?: "default" | "secondary" | "outline" | "success" | "destructive";
  /** Typography weight selected without competing utility declarations. */
  weight?: "normal" | "medium";
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
        "flex-none whitespace-nowrap px-2 py-0.5 rounded-md border text-xs",
        props.weight === "normal" ? "font-normal" : "font-medium",
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
    <Badge
      variant={variant()}
      weight="normal"
      class={join("font-mono", props.class)}
    >
      {value()}
      {props.label === "" ? "" : ` ${props.label ?? "fps"}`}
    </Badge>
  );
}

export function Card(props: {
  children?: JSX.Element;
  class?: string;
  ref?: ViewProps["ref"];
  shadows?: readonly import("@wabou/core/style").Shadow[] | null;
  role?: ViewProps["role"];
  "aria-label"?: string;
  "aria-hidden"?: ViewProps["aria-hidden"];
}): JSX.Element {
  const theme = useComponentsTheme();
  return (
    <View
      ref={props.ref}
      role={props.role}
      aria-label={props["aria-label"]}
      aria-hidden={props["aria-hidden"]}
      class={join(
        "min-w-0 min-h-0 flex flex-col overflow-hidden rounded-lg border",
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
    <View class={join("min-w-0 flex flex-col gap-1 px-4 pt-4", props.class)}>
      {props.children}
    </View>
  );
}
export interface CardTitleProps extends TextProps {}

export function CardTitle(props: CardTitleProps): JSX.Element {
  return (
    <Text
      {...props}
      class={join(
        "min-w-0 text-base font-semibold",
        "text-primary",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}
export interface CardDescriptionProps extends TextProps {}

export function CardDescription(props: CardDescriptionProps): JSX.Element {
  return (
    <Text
      {...props}
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
    <View class={join("min-w-0 min-h-0 flex flex-col gap-3 p-4", props.class)}>
      {props.children}
    </View>
  );
}
export function CardFooter(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <View
      class={join("min-w-0 flex items-center gap-2 px-4 pb-4", props.class)}
    >
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
  class?: string;
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
    <View class={join("w-full min-w-0 flex items-center gap-3", props.class)}>
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
      {props.label && (
        <Text class="min-w-0 flex-1 whitespace-normal text-sm text-secondary">
          {props.label}
        </Text>
      )}
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
