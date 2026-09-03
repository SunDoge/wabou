import { mergeClasses } from "@wabou/core/style";
import { createSignal, type JSX } from "solid-js";
import { NativeWidget } from "../primitives";
import { decimalPlaces, finiteOr, normalizeRange } from "./range";
import { componentsDisabledInteractiveClass } from "./theme";

interface SliderKeyEvent {
  key: string;
  preventDefault(): void;
}

export interface SliderProps {
  value?: number;
  defaultValue?: number;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  /** Direction of the track and pointer interaction. */
  orientation?: "horizontal" | "vertical";
  /** Fill from the thumb toward the maximum end without changing values. */
  reversed?: boolean;
  label: string;
  valueText?: (value: number) => string;
  onValueChange?: (value: number) => void;
  class?: string;
}

export function Slider(props: SliderProps): JSX.Element {
  const orientation = () => props.orientation ?? "horizontal";
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

  return (
    <NativeWidget
      tag="slider"
      role="slider"
      aria-label={props.label}
      aria-valuemin={min()}
      aria-valuemax={max()}
      aria-valuenow={value()}
      aria-valuetext={props.valueText?.(value()) ?? String(value())}
      aria-disabled={props.disabled ?? false}
      aria-orientation={orientation()}
      focusOrder={props.disabled ? -1 : 0}
      class={mergeClasses(
        orientation() === "vertical"
          ? "w-7 h-[120px] select-none"
          : "w-full h-7 select-none",
        props.disabled ? "" : "cursor-pointer",
        props.class,
        componentsDisabledInteractiveClass(props.disabled ?? false),
      )}
      config={{
        min: min(),
        max: max(),
        step: step(),
        value: value(),
        disabled: props.disabled ?? false,
        orientation: orientation(),
        reversed: props.reversed ?? false,
      }}
      onChange={(event) => update(event.value)}
      onKeyDown={onKeyDown}
    />
  );
}
