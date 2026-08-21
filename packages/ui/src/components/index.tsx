import { createFps } from "@wabou/core/renderer";
import { createSignal, type JSX } from "solid-js";
import { match, P } from "ts-pattern";
import { createTransition, useReducedMotion } from "../animation";
import {
  type ButtonState,
  Button as HeadlessButton,
  PasswordInput as PrimitivePasswordInput,
  type PasswordInputProps as PrimitivePasswordInputProps,
  TextArea as PrimitiveTextArea,
  type TextAreaProps as PrimitiveTextAreaProps,
  Text,
  translate2d,
  View,
} from "../primitives";
import { Badge, type BadgeProps } from "./badge";
import { join } from "./class-names";

export * from "./alert";
export * from "./alert-dialog";
export * from "./aspect-ratio";
export * from "./attachment";
export * from "./avatar";
export * from "./badge";
export * from "./button";
export * from "./button-group";
export * from "./card";
export * from "./carousel";
export * from "./chart";
export * from "./code-block";
export * from "./combobox";
export * from "./command";
export * from "./config-editor";
export * from "./context-menu";
export * from "./copy-button";
export * from "./data-table";
export * from "./date-picker";
export * from "./dialog";
export * from "./direction";
export * from "./directory-picker";
export * from "./disclosure";
export {
  Kbd,
  KbdGroup,
  Skeleton,
  type SkeletonProps,
  Spinner,
} from "./display";
export * from "./drawer";
export * from "./dropdown-menu";
export * from "./empty";
export * from "./forms";
export * from "./hover-card";
export * from "./input";
export * from "./input-otp";
export * from "./item";
export * from "./label";
export * from "./layout";
export * from "./menubar";
export * from "./message";
export * from "./message-scroller";
export * from "./native-select";
export * from "./navigation";
export * from "./navigation-menu";
export * from "./number-field";
export * from "./page";
export * from "./popover";
export * from "./progress";
export * from "./rating";
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
export * from "./separator";
export * from "./sheet";
export * from "./sidebar";
export * from "./slider";
export * from "./split-button";
export * from "./stat-card";
export * from "./stepper";
export * from "./table";
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
export * from "./timeline";
export * from "./title-bar";
export * from "./toast";
export * from "./toolbar";
export * from "./tooltip";
export * from "./tree-view";
export * from "./typography";

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
  const reducedMotion = useReducedMotion();
  const movement = createTransition(() => (checked() ? 20 : 0), {
    duration: 0.18,
    ease: [0.22, 1, 0.36, 1],
    reducedMotion,
  });
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
          transform={translate2d(movement.value(), 0)}
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
