import { NumberFormatter, NumberParser } from "@internationalized/number";
import { useHost } from "@wabou/core";
import minus from "lucide-static/icons/minus.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import {
  createEffect,
  createMemo,
  createSignal,
  type JSX,
  omit,
  untrack,
} from "solid-js";
import { match } from "ts-pattern";
import { Icon } from "../primitives";
import { InputGroup, InputGroupButton, InputGroupInput } from "./forms";
import type { InputProps } from "./input";
import {
  addNumberFieldStep,
  clampNumberFieldValue,
  normalizeNumberFieldRange,
  numberFieldValueFromEmpty,
} from "./number-field-state";
import { createControllableState } from "./state";

export interface NumberFieldProps
  extends Omit<InputProps, "class" | "onInput" | "placeholder" | "value"> {
  /** `undefined` selects uncontrolled mode; `null` represents an empty field. */
  value?: number | null;
  defaultValue?: number | null;
  min?: number;
  max?: number;
  step?: number;
  largeStep?: number;
  locale?: string;
  formatOptions?: Intl.NumberFormatOptions;
  placeholder?: string;
  changeOnWheel?: boolean;
  onValueChange?(value: number | null): void;
  class?: string;
  inputClass?: string;
  incrementLabel?: string;
  decrementLabel?: string;
  "aria-label": string;
}

/** Locale-aware numeric input with explicit native stepping semantics. */
export function NumberField(props: NumberFieldProps): JSX.Element {
  const host = useHost();
  const forwarded = omit(
    props,
    "value",
    "defaultValue",
    "min",
    "max",
    "step",
    "largeStep",
    "locale",
    "formatOptions",
    "placeholder",
    "changeOnWheel",
    "onValueChange",
    "class",
    "inputClass",
    "incrementLabel",
    "decrementLabel",
  );
  const range = () =>
    normalizeNumberFieldRange(
      props.min,
      props.max,
      props.step,
      props.largeStep,
    );
  const locale = () => props.locale ?? host.intl.locale();
  const parser = createMemo(
    () => new NumberParser(locale(), props.formatOptions),
  );
  const formatter = createMemo(
    () => new NumberFormatter(locale(), props.formatOptions),
  );
  const state = createControllableState<number | null>({
    value: () => props.value,
    defaultValue: props.defaultValue ?? null,
    disabled: () => Boolean(props.disabled || props.readOnly),
    onChange: props.onValueChange,
  });
  const formattedValue = () => {
    const value = state.value();
    return value === null ? "" : formatter().format(value);
  };
  const [focused, setFocused] = createSignal(false);
  const [draft, setDraft] = createSignal(untrack(formattedValue));

  createEffect(
    () => ({ focused: focused(), value: formattedValue() }),
    ({ focused: isFocused, value }) => {
      if (!isFocused) setDraft(value);
    },
  );

  const update = (next: number | null) => {
    if (props.disabled || props.readOnly) return false;
    const normalized =
      next === null ? null : clampNumberFieldValue(next, range());
    const changed = state.set(normalized);
    setDraft(formattedValue());
    return changed;
  };
  const commitDraft = () => {
    const value = draft().trim();
    if (!value) {
      update(null);
      return;
    }
    const parsed = parser().parse(value);
    if (Number.isFinite(parsed)) update(parsed);
    else setDraft(formattedValue());
  };
  const changeBy = (direction: -1 | 1, amount: number) => {
    const current = state.value();
    const next =
      current === null
        ? numberFieldValueFromEmpty(direction, range())
        : addNumberFieldStep(current, direction * amount, range());
    update(next);
  };
  const canDecrement = () => {
    const value = state.value();
    return (
      !props.disabled &&
      !props.readOnly &&
      (value === null || value > range().min)
    );
  };
  const canIncrement = () => {
    const value = state.value();
    return (
      !props.disabled &&
      !props.readOnly &&
      (value === null || value < range().max)
    );
  };

  return (
    <InputGroup class={props.class}>
      <InputGroupButton
        size="icon"
        class="w-7 h-7 mx-0.5"
        disabled={!canDecrement()}
        aria-label={props.decrementLabel ?? `Decrease ${props["aria-label"]}`}
        onClick={() => changeBy(-1, range().step)}
      >
        <Icon source={minus} aria-hidden="true" size={14} />
      </InputGroupButton>
      <InputGroupInput
        {...forwarded}
        role="spinbutton"
        aria-label={props["aria-label"]}
        aria-valuemin={Number.isFinite(range().min) ? range().min : undefined}
        aria-valuemax={Number.isFinite(range().max) ? range().max : undefined}
        aria-valuenow={state.value() ?? undefined}
        aria-valuetext={state.value() === null ? undefined : formattedValue()}
        value={draft()}
        placeholder={props.placeholder}
        class={props.inputClass}
        onFocus={(event) => {
          setFocused(true);
          props.onFocus?.(event);
        }}
        onBlur={(event) => {
          commitDraft();
          setFocused(false);
          props.onBlur?.(event);
        }}
        onInput={(event) => {
          const next = event.currentTarget.value;
          // Keep the native editor and Solid draft in sync even for invalid
          // text. Only valid partial numbers update the canonical value; blur
          // restores the formatted canonical value otherwise.
          setDraft(next);
          if (!parser().isValidPartialNumber(next, range().min, range().max))
            return;
          const parsed = parser().parse(next);
          if (
            Number.isFinite(parsed) &&
            parsed >= range().min &&
            parsed <= range().max
          ) {
            state.set(parsed);
          }
        }}
        onKeyDown={(event) => {
          props.onKeyDown?.(event);
          if (event.defaultPrevented || props.disabled || props.readOnly)
            return;
          const handled = match(event.key)
            .with("ArrowUp", () => changeBy(1, range().step))
            .with("ArrowDown", () => changeBy(-1, range().step))
            .with("PageUp", () => changeBy(1, range().largeStep))
            .with("PageDown", () => changeBy(-1, range().largeStep))
            .with("Home", () =>
              Number.isFinite(range().min) ? update(range().min) : false,
            )
            .with("End", () =>
              Number.isFinite(range().max) ? update(range().max) : false,
            )
            .otherwise(() => false);
          if (handled) event.preventDefault();
        }}
        onWheel={(event) => {
          props.onWheel?.(event);
          if (
            event.defaultPrevented ||
            !props.changeOnWheel ||
            !focused() ||
            props.disabled ||
            props.readOnly ||
            event.deltaY === 0
          )
            return;
          changeBy(event.deltaY < 0 ? 1 : -1, range().step);
          event.preventDefault();
        }}
      />
      <InputGroupButton
        size="icon"
        class="w-7 h-7 mx-0.5"
        disabled={!canIncrement()}
        aria-label={props.incrementLabel ?? `Increase ${props["aria-label"]}`}
        onClick={() => changeBy(1, range().step)}
      >
        <Icon source={plus} aria-hidden="true" size={14} />
      </InputGroupButton>
    </InputGroup>
  );
}
