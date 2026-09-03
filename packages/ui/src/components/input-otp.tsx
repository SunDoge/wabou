import { mergeClasses } from "@wabou/core/style";
import minus from "lucide-static/icons/minus.svg?raw";
import {
  createContext,
  createSignal,
  type JSX,
  omit,
  Show,
  useContext,
} from "solid-js";
import {
  Icon,
  Text,
  TextInput,
  type TextInputProps,
  View,
  type ViewProps,
} from "../primitives";
import { componentsDisabledInteractiveClass } from "./theme";

export function normalizeOtpValue(
  value: string,
  maxLength: number,
  allowed: RegExp = /^[0-9]$/,
): string {
  if (!Number.isInteger(maxLength) || maxLength <= 0) {
    throw new RangeError("InputOTP maxLength must be a positive integer");
  }
  return Array.from(value)
    .filter((character) => {
      allowed.lastIndex = 0;
      return allowed.test(character);
    })
    .slice(0, maxLength)
    .join("");
}

interface InputOtpContextValue {
  value(): string;
  maxLength(): number;
  focused(): boolean;
}

const InputOtpContext = createContext<InputOtpContextValue>();

function requireInputOtp(): InputOtpContextValue {
  const context = useContext(InputOtpContext);
  if (!context) throw new Error("InputOTPSlot requires an InputOTP root");
  return context;
}

export interface InputOTPProps {
  value?: string;
  defaultValue?: string;
  maxLength: number;
  allowed?: RegExp;
  disabled?: boolean;
  readOnly?: boolean;
  "aria-label": string;
  class?: string;
  inputClass?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
  onComplete?: (value: string) => void;
  ref?: TextInputProps["ref"];
}

export function InputOTP(props: InputOTPProps): JSX.Element {
  const initial = normalizeOtpValue(
    props.defaultValue ?? "",
    props.maxLength,
    props.allowed,
  );
  const [internalValue, setInternalValue] = createSignal(initial);
  const [focused, setFocused] = createSignal(false);
  const value = () =>
    normalizeOtpValue(
      props.value ?? internalValue(),
      props.maxLength,
      props.allowed,
    );
  const context: InputOtpContextValue = {
    value,
    maxLength: () => props.maxLength,
    focused,
  };

  return (
    <InputOtpContext value={context}>
      <View
        role="group"
        aria-label={props["aria-label"]}
        class={mergeClasses(
          "relative inline-flex flex-none items-center gap-2",
          props.class,
          componentsDisabledInteractiveClass(props.disabled ?? false),
        )}
      >
        {props.children}
        <TextInput
          ref={props.ref}
          aria-label={props["aria-label"]}
          value={value()}
          disabled={props.disabled}
          readOnly={props.readOnly}
          class={mergeClasses(
            "absolute inset-0 w-full h-full z-10 opacity-0",
            props.inputClass,
          )}
          onFocus={() => setFocused(true)}
          onBlur={() => setFocused(false)}
          onInput={(event) => {
            const previous = value();
            const next = normalizeOtpValue(
              event.currentTarget.value,
              props.maxLength,
              props.allowed,
            );
            setInternalValue(next);
            props.onValueChange?.(next);
            if (
              next.length === props.maxLength &&
              previous.length !== props.maxLength
            ) {
              props.onComplete?.(next);
            }
          }}
        />
      </View>
    </InputOtpContext>
  );
}

export function InputOTPGroup(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      aria-hidden="true"
      class={mergeClasses("flex flex-none items-center gap-1", props.class)}
    >
      {props.children}
    </View>
  );
}

export interface InputOTPSlotProps extends Omit<ViewProps, "children"> {
  index: number;
}

export function InputOTPSlot(props: InputOTPSlotProps): JSX.Element {
  const context = requireInputOtp();
  const forwarded = omit(props, "index", "class");
  const character = () => context.value()[props.index];
  const active = () =>
    context.focused() &&
    props.index === Math.min(context.value().length, context.maxLength() - 1);
  return (
    <View
      {...forwarded}
      class={mergeClasses(
        "relative w-9 h-9 flex-none flex items-center justify-center rounded-lg border bg-input text-sm text-primary shadow-xs",
        active() ? "border-focus" : "border-subtle",
        props.class,
      )}
    >
      <Show when={character()} keyed>
        {(value) => <Text class="text-sm text-primary">{value}</Text>}
      </Show>
      <Show when={active() && !character()}>
        <View
          aria-hidden="true"
          class="absolute w-px h-4 bg-primary pointer-events-none"
        />
      </Show>
    </View>
  );
}

export function InputOTPSeparator(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      aria-hidden="true"
      class={mergeClasses(
        "w-5 h-9 flex-none flex items-center justify-center",
        props.class,
      )}
    >
      <Icon source={minus} size={14} class="text-muted" />
    </View>
  );
}
