import { createElement, spread } from "@wabou/core/renderer";
import { createSignal, type JSX } from "solid-js";
import { mergeClasses } from "@wabou/core/style";
import { decimalPlaces, finiteOr, normalizeRange } from "./range";

interface SliderKeyEvent {
  key: string;
  preventDefault(): void;
}

interface SliderChangeEvent {
  value: number;
}

export interface SliderProps {
  value?: number;
  defaultValue?: number;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  label: string;
  valueText?: (value: number) => string;
  onValueChange?: (value: number) => void;
  class?: string;
}

export function Slider(props: SliderProps): JSX.Element {
  const range = () => normalizeRange(props.min, props.max, props.step);
  const min = () => range().min;
  const max = () => range().max;
  const step = () => range().step;
  const clamp = (value: number) =>
    Math.max(min(), Math.min(max(), finiteOr(value, min())));
  const snap = (value: number) => {
    const stepped =
      min() + Math.round((clamp(value) - min()) / step()) * step();
    const precision = decimalPlaces(step());
    return clamp(Number(stepped.toFixed(precision)));
  };
  const [local, setLocal] = createSignal(snap(props.defaultValue ?? min()));
  const value = () => snap(props.value ?? local());

  const update = (next: number) => {
    if (props.disabled) return;
    const normalized = snap(next);
    const changed = normalized !== value();
    if (props.value === undefined) setLocal(normalized);
    if (changed) props.onValueChange?.(normalized);
  };
  const changeBy = (amount: number) => update(value() + amount);
  const onKeyDown = (event: SliderKeyEvent) => {
    if (props.disabled) return;
    if (event.key === "ArrowLeft" || event.key === "ArrowDown")
      changeBy(-step());
    else if (event.key === "ArrowRight" || event.key === "ArrowUp")
      changeBy(step());
    else if (event.key === "PageDown") changeBy(-step() * 10);
    else if (event.key === "PageUp") changeBy(step() * 10);
    else if (event.key === "Home") update(min());
    else if (event.key === "End") update(max());
    else return;
    event.preventDefault();
  };

  const node = createElement("slider");
  spread(
    node,
    {
      role: "slider",
      get "aria-label"() {
        return props.label;
      },
      get "aria-valuemin"() {
        return min();
      },
      get "aria-valuemax"() {
        return max();
      },
      get "aria-valuenow"() {
        return value();
      },
      get "aria-valuetext"() {
        return props.valueText?.(value()) ?? String(value());
      },
      get "aria-disabled"() {
        return props.disabled ?? false;
      },
      get focusOrder() {
        return props.disabled ? -1 : 0;
      },
      get class() {
        return mergeClasses(
          "h-7 select-none",
          props.disabled ? "cursor-not-allowed" : "cursor-pointer",
          props.class,
        );
      },
      get widgetConfig() {
        return {
          min: min(),
          max: max(),
          step: step(),
          value: value(),
          disabled: props.disabled ?? false,
        };
      },
      onChange: (event: SliderChangeEvent) => update(event.value),
      onKeyDown,
    },
    false,
  );
  return node as unknown as JSX.Element;
}
