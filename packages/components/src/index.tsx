import { type AnimationControls, animate } from "@wabou/animation";
import {
  Button as HeadlessButton,
  type ButtonProps as HeadlessButtonProps,
  TextArea as PrimitiveTextArea,
  type TextAreaProps as PrimitiveTextAreaProps,
  Text,
  translate2d,
  View,
  type WabouStyle,
} from "@wabou/primitives";
import { createFps } from "@wabou/solid-renderer";
import {
  createEffect,
  createSignal,
  type JSX,
  onCleanup,
  splitProps,
  untrack,
} from "solid-js";
import { type ComponentsTheme, useComponentsTheme } from "./theme";

export {
  ComponentsProvider,
  type ComponentsProviderProps,
  type ComponentsTheme,
  useComponentsTheme,
} from "./theme";

const join = (...values: Array<string | undefined | false>) =>
  values.filter(Boolean).join(" ");

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

function buttonBackground(
  variant: ButtonVariant,
  hovered: boolean,
  pressed: boolean,
  theme: ComponentsTheme,
) {
  if (variant === "default")
    return pressed ? "#0369a1" : hovered ? "#0284c7" : "#0ea5e9";
  if (variant === "secondary") {
    if (theme === "light")
      return pressed ? "#cbd5e1" : hovered ? "#e2e8f0" : "#f1f5f9";
    return pressed ? "#334155" : hovered ? "#475569" : "#334155";
  }
  if (variant === "destructive")
    return pressed ? "#b91c1c" : hovered ? "#dc2626" : "#ef4444";
  if (variant === "outline") {
    if (theme === "light")
      return pressed ? "#e2e8f0" : hovered ? "#f1f5f9" : "transparent";
    return pressed ? "#1e293b" : hovered ? "#334155" : "transparent";
  }
  return hovered || pressed
    ? theme === "light"
      ? "#f1f5f9"
      : "#1e293b"
    : "transparent";
}

function buttonBorder(
  variant: ButtonVariant,
  focused: boolean,
  theme: ComponentsTheme,
) {
  if (focused) return "#7dd3fc";
  return variant === "outline"
    ? theme === "light"
      ? "#cbd5e1"
      : "#475569"
    : "transparent";
}

export function Button(props: ButtonProps): JSX.Element {
  const theme = useComponentsTheme();
  const [local, forwarded] = splitProps(props, [
    "variant",
    "size",
    "class",
    "style",
  ]);
  const variant = () => local.variant ?? "default";
  const size = () => local.size ?? "default";
  return (
    <HeadlessButton
      {...forwarded}
      unstyled
      class={join(
        "inline-flex flex-none whitespace-nowrap items-center justify-center rounded-md border font-medium",
        variant() === "secondary"
          ? theme() === "dark"
            ? "border-transparent text-slate-100"
            : "border-transparent text-slate-900"
          : variant() === "outline"
            ? theme() === "dark"
              ? "border-slate-600 text-slate-100"
              : "border-slate-300 text-slate-900"
            : variant() === "ghost"
              ? theme() === "dark"
                ? "border-transparent text-slate-200"
                : "border-transparent text-slate-700"
              : "border-transparent text-white",
        size() === "sm"
          ? "h-8 px-3 text-xs"
          : size() === "lg"
            ? "h-10 px-6 text-sm"
            : size() === "icon"
              ? "w-9 h-9 p-0 text-sm"
              : "h-9 px-4 text-sm",
        local.class,
      )}
      style={(state) =>
        ({
          "background-color": buttonBackground(
            variant(),
            state.hovered,
            state.pressed,
            theme(),
          ),
          "border-color": buttonBorder(variant(), state.focused, theme()),
          "border-width": state.focused ? 2 : 1,
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

export function Badge(props: BadgeProps): JSX.Element {
  const theme = useComponentsTheme();
  return (
    <Text
      class={join(
        "flex-none whitespace-nowrap px-2 py-1 rounded-full border text-xs font-medium",
        props.variant === "secondary"
          ? theme() === "dark"
            ? "bg-slate-700 border-slate-600 text-slate-100"
            : "bg-slate-100 border-slate-200 text-slate-700"
          : props.variant === "outline"
            ? theme() === "dark"
              ? "bg-transparent border-slate-600 text-slate-300"
              : "bg-transparent border-slate-300 text-slate-600"
            : props.variant === "success"
              ? theme() === "dark"
                ? "bg-emerald-700 border-emerald-600 text-emerald-50"
                : "bg-emerald-100 border-emerald-200 text-emerald-800"
              : props.variant === "destructive"
                ? theme() === "dark"
                  ? "bg-red-700 border-red-600 text-red-50"
                  : "bg-red-100 border-red-200 text-red-800"
                : "bg-sky-600 border-sky-500 text-white",
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
  const variant = (): BadgeProps["variant"] => {
    if (value() === 0) return "outline";
    if (value() >= (props.goodAt ?? 55)) return "success";
    if (value() < (props.warningBelow ?? 30)) return "destructive";
    return "secondary";
  };
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
}): JSX.Element {
  const theme = useComponentsTheme();
  return (
    <View
      class={join(
        "flex flex-col rounded-xl border",
        theme() === "dark"
          ? "border-slate-700 bg-slate-900"
          : "border-slate-200 bg-white",
        props.class,
      )}
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
    <View class={join("flex flex-col gap-1 px-5 pt-5", props.class)}>
      {props.children}
    </View>
  );
}
export function CardTitle(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  const theme = useComponentsTheme();
  return (
    <Text
      class={join(
        "text-base font-semibold",
        theme() === "dark" ? "text-slate-50" : "text-slate-950",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}
export function CardDescription(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  const theme = useComponentsTheme();
  return (
    <Text
      class={join(
        "text-sm",
        theme() === "dark" ? "text-slate-400" : "text-slate-500",
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
    <View class={join("flex flex-col gap-3 p-5", props.class)}>
      {props.children}
    </View>
  );
}
export function CardFooter(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <View class={join("flex items-center gap-2 px-5 pb-5", props.class)}>
      {props.children}
    </View>
  );
}

export function Separator(props: {
  orientation?: "horizontal" | "vertical";
  class?: string;
}): JSX.Element {
  const theme = useComponentsTheme();
  return (
    <View
      class={join(
        "flex-none",
        theme() === "dark" ? "bg-slate-700" : "bg-slate-200",
        props.orientation === "vertical" ? "w-px h-full" : "h-px w-full",
        props.class,
      )}
    />
  );
}

export function Alert(props: {
  title: string;
  children?: JSX.Element;
  variant?: "default" | "destructive";
  class?: string;
}): JSX.Element {
  const destructive = props.variant === "destructive";
  const theme = useComponentsTheme();
  return (
    <View
      class={join(
        "flex flex-col gap-1 rounded-lg border p-4",
        destructive
          ? theme() === "dark"
            ? "border-red-800 bg-red-950"
            : "border-red-200 bg-red-50"
          : theme() === "dark"
            ? "border-slate-700 bg-slate-900"
            : "border-slate-200 bg-white",
        props.class,
      )}
    >
      <Text
        class={join(
          "text-sm font-semibold",
          destructive
            ? theme() === "dark"
              ? "text-red-200"
              : "text-red-900"
            : theme() === "dark"
              ? "text-slate-100"
              : "text-slate-950",
        )}
      >
        {props.title}
      </Text>
      <Text
        class={join(
          "text-sm",
          destructive
            ? theme() === "dark"
              ? "text-red-300"
              : "text-red-700"
            : theme() === "dark"
              ? "text-slate-400"
              : "text-slate-600",
        )}
      >
        {props.children}
      </Text>
    </View>
  );
}

export interface InputProps {
  value?: string;
  placeholder?: string;
  disabled?: boolean;
  class?: string;
  onInput?: (event: { currentTarget: { value: string } }) => void;
}
export function Input(props: InputProps): JSX.Element {
  const theme = useComponentsTheme();
  return (
    <input
      {...props}
      class={join(
        "h-9 w-full px-3 rounded-md border text-sm",
        theme() === "dark"
          ? "border-slate-600 bg-slate-950 text-slate-100"
          : "border-slate-300 bg-white text-slate-900",
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
  const theme = useComponentsTheme();
  return (
    <PrimitiveTextArea
      {...props}
      class={join(
        "h-24 w-full px-3 py-2 rounded-md border text-sm",
        theme() === "dark"
          ? "border-slate-600 bg-slate-950 text-slate-100"
          : "border-slate-300 bg-white text-slate-900",
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
}
export function Switch(props: SwitchProps): JSX.Element {
  const theme = useComponentsTheme();
  const [local, setLocal] = createSignal(props.defaultChecked ?? false);
  const checked = () => props.checked ?? local();
  const [thumbX, setThumbX] = createSignal(checked() ? 20 : 0);
  let movement: AnimationControls | undefined;
  createEffect(() => {
    const target = checked() ? 20 : 0;
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
        aria-checked={checked()}
        class="w-11 h-6 flex-none rounded-full p-0.5"
        style={(state) => ({
          "background-color": checked()
            ? state.pressed
              ? "#0369a1"
              : state.hovered
                ? "#0284c7"
                : "#0ea5e9"
            : state.hovered
              ? theme() === "dark"
                ? "#475569"
                : "#cbd5e1"
              : theme() === "dark"
                ? "#334155"
                : "#e2e8f0",
          opacity: state.disabled ? 0.45 : 1,
        })}
        onClick={toggle}
      >
        <View
          class="w-5 h-5 rounded-full"
          transform={translate2d(thumbX(), 0)}
          style={{
            "background-color": checked()
              ? "#ffffff"
              : theme() === "dark"
                ? "#e2e8f0"
                : "#ffffff",
          }}
        />
      </HeadlessButton>
      {props.label && (
        <Text
          class={
            theme() === "dark"
              ? "text-sm text-slate-200"
              : "text-sm text-slate-700"
          }
        >
          {props.label}
        </Text>
      )}
    </View>
  );
}

export function Progress(props: {
  value?: number;
  class?: string;
}): JSX.Element {
  const theme = useComponentsTheme();
  const value = () => Math.max(0, Math.min(100, props.value ?? 0));
  return (
    <View
      class={join(
        "w-full h-2 overflow-hidden rounded-full",
        theme() === "dark" ? "bg-slate-700" : "bg-slate-200",
        props.class,
      )}
    >
      <View
        class="h-full bg-sky-500 rounded-full"
        style={{ width: `${value()}%` }}
      />
    </View>
  );
}
