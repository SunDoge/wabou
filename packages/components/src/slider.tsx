import { createMeasuredSize, View } from "@wabou/primitives";
import { createSignal, type JSX } from "solid-js";
import { join } from "./class-names";
import { decimalPlaces, finiteOr, normalizeRange } from "./range";

interface SliderPointerEvent {
  offsetX: number;
  buttons: number;
  preventDefault(): void;
}

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
  const ratio = () =>
    max() === min() ? 0 : (value() - min()) / (max() - min());
  const measured = createMeasuredSize();
  const [dragging, setDragging] = createSignal(false);
  const [focused, setFocused] = createSignal(false);

  const update = (next: number) => {
    if (props.disabled) return;
    const normalized = snap(next);
    const changed = normalized !== value();
    if (props.value === undefined) setLocal(normalized);
    if (changed) props.onValueChange?.(normalized);
  };
  const updateFromPointer = (event: SliderPointerEvent) => {
    const width = measured.width();
    if (width <= 0) return;
    event.preventDefault();
    update(
      min() +
        (Math.max(0, Math.min(width, event.offsetX)) / width) * (max() - min()),
    );
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
    <View
      ref={measured.ref}
      role="slider"
      aria-label={props.label}
      aria-valuemin={min()}
      aria-valuemax={max()}
      aria-valuenow={value()}
      aria-valuetext={props.valueText?.(value()) ?? String(value())}
      aria-disabled={props.disabled}
      focusOrder={props.disabled ? -1 : 0}
      class={join(
        "h-7 relative flex items-center",
        props.disabled ? "cursor-not-allowed" : "cursor-pointer",
        props.class,
      )}
      onFocus={() => setFocused(true)}
      onBlur={() => {
        setFocused(false);
        setDragging(false);
      }}
      onPointerDown={(event: SliderPointerEvent) => {
        setDragging(true);
        updateFromPointer(event);
      }}
      onPointerMove={(event: SliderPointerEvent) => {
        if (dragging() && event.buttons !== 0) updateFromPointer(event);
      }}
      onPointerUp={(event: SliderPointerEvent) => {
        if (dragging()) updateFromPointer(event);
        setDragging(false);
      }}
      onPointerCancel={() => setDragging(false)}
      onKeyDown={onKeyDown}
      style={{ opacity: props.disabled ? 0.45 : 1 }}
    >
      <View
        aria-hidden="true"
        class="w-full h-1.5 overflow-hidden rounded-full border border-subtle bg-control"
      >
        <View
          class="h-full rounded-full bg-accent"
          style={{ width: `${ratio() * 100}%` }}
        />
      </View>
      <View
        aria-hidden="true"
        class={join(
          "w-4 h-4 absolute rounded-full border bg-surface shadow-xs",
          focused() || dragging() ? "border-focus" : "border-strong",
        )}
        style={{
          left: `${ratio() * Math.max(0, measured.width() - 16)}px`,
          top: "6px",
        }}
      />
    </View>
  );
}
