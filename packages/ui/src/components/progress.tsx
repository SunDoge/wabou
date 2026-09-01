import { mergeClasses } from "@wabou/core/style";
import {
  type Accessor,
  createContext,
  createMemo,
  createSignal,
  type JSX,
  omit,
  Show,
  useContext,
} from "solid-js";
import { createSweep, useReducedMotion } from "../animation";
import {
  createMeasuredSize,
  Svg,
  Text,
  type TextProps,
  View,
  type ViewProps,
} from "../primitives";
import { Spinner } from "./display";
import { progressCircleSource } from "./progress-circle-source";
import { finiteOr } from "./range";

export interface ProgressValueDetails {
  value: number;
  min: number;
  max: number;
  percent: number;
}

export interface ProgressRootProps
  extends Omit<ViewProps, "children" | "class" | "role"> {
  value?: number;
  minValue?: number;
  maxValue?: number;
  indeterminate?: boolean;
  /** Direct accessible label; visual labels do not rely on DOM id wiring. */
  label?: string;
  getValueLabel?: (details: ProgressValueDetails) => string;
  children?: JSX.Element;
  class?: string;
}

export type ProgressSize = "xs" | "sm" | "default" | "lg";

interface ProgressContextValue {
  value: Accessor<number>;
  min: Accessor<number>;
  max: Accessor<number>;
  percent: Accessor<number>;
  indeterminate: Accessor<boolean>;
  label: Accessor<string>;
  valueLabel: Accessor<string | undefined>;
  trackWidth: Accessor<number>;
  setTrackWidth(width: number): void;
}

const ProgressContext = createContext<ProgressContextValue>();

function useProgressContext(): ProgressContextValue {
  const context = useContext(ProgressContext);
  if (!context)
    throw new Error("Progress parts must be used inside ProgressRoot");
  return context;
}

export function normalizeProgressValue(
  value: number | undefined,
  minValue: number | undefined,
  maxValue: number | undefined,
): ProgressValueDetails {
  const min = finiteOr(minValue, 0);
  const requestedMax = finiteOr(maxValue, 100);
  const max = requestedMax > min ? requestedMax : min + 1;
  const normalizedValue = Math.max(min, Math.min(max, finiteOr(value, min)));
  return {
    value: normalizedValue,
    min,
    max,
    percent: ((normalizedValue - min) / (max - min)) * 100,
  };
}

/** Semantic progress state with explicit, composable visual parts. */
export function ProgressRoot(props: ProgressRootProps): JSX.Element {
  const [trackWidth, setTrackWidth] = createSignal(0, { ownedWrite: true });
  const forwarded = omit(
    props,
    "value",
    "minValue",
    "maxValue",
    "indeterminate",
    "label",
    "getValueLabel",
    "children",
    "class",
  );
  const details = () =>
    normalizeProgressValue(props.value, props.minValue, props.maxValue);
  const indeterminate = () => props.indeterminate ?? false;
  const defaultValueLabel = () => `${Math.round(details().percent)} percent`;
  const context: ProgressContextValue = {
    value: () => details().value,
    min: () => details().min,
    max: () => details().max,
    percent: () => details().percent,
    indeterminate,
    label: () => props.label ?? "Progress",
    valueLabel: () =>
      indeterminate()
        ? undefined
        : (props.getValueLabel?.(details()) ?? defaultValueLabel()),
    trackWidth,
    setTrackWidth,
  };

  return (
    <ProgressContext value={context}>
      <View
        {...forwarded}
        role="progressbar"
        aria-label={context.label()}
        aria-valuemin={context.min()}
        aria-valuemax={context.max()}
        aria-valuenow={indeterminate() ? undefined : context.value()}
        aria-valuetext={context.valueLabel()}
        class={mergeClasses("w-full min-w-0 flex flex-col gap-2", props.class)}
      >
        {props.children}
      </View>
    </ProgressContext>
  );
}

export interface ProgressTrackProps extends ViewProps {
  size?: ProgressSize;
}

const progressTrackSize = (size: ProgressSize | undefined) =>
  size === "xs"
    ? "h-1"
    : size === "sm"
      ? "h-1.5"
      : size === "lg"
        ? "h-2.5"
        : "h-2";

export function ProgressTrack(props: ProgressTrackProps): JSX.Element {
  const context = useProgressContext();
  const forwarded = omit(props, "size");
  const measured = createMeasuredSize({
    onChange: ({ width }) => context.setTrackWidth(width),
  });
  return (
    <View
      {...forwarded}
      ref={(node) => {
        measured.ref(node);
        props.ref?.(node);
      }}
      aria-hidden="true"
      class={mergeClasses(
        "w-full flex-none overflow-hidden rounded-full bg-control",
        progressTrackSize(props.size),
        props.class,
      )}
    />
  );
}

function IndeterminateProgressFill(props: ViewProps): JSX.Element {
  const context = useProgressContext();
  const reducedMotion = useReducedMotion();
  const sweep = createSweep({
    extent: context.trackWidth,
    itemRatio: 0.4,
    duration: 1.35,
    ease: "easeInOut",
    reducedMotion,
    reducedValue: 0.5,
  });
  return (
    <View
      {...props}
      aria-hidden="true"
      class={mergeClasses("w-2/5 h-full rounded-full bg-accent", props.class)}
      transform={sweep.transform()}
    />
  );
}

export function ProgressFill(props: ViewProps): JSX.Element {
  const context = useProgressContext();
  return (
    <Show
      when={!context.indeterminate()}
      fallback={<IndeterminateProgressFill {...props} />}
    >
      <View
        {...props}
        aria-hidden="true"
        class={mergeClasses(
          "h-full rounded-full bg-accent",
          context.percent() < 100 && "rounded-r-none",
          props.class,
        )}
        style={{ width: `${context.percent()}%`, ...props.style }}
      />
    </Show>
  );
}

export function ProgressLabel(props: TextProps): JSX.Element {
  const context = useProgressContext();
  return (
    <Text
      {...props}
      class={mergeClasses("min-w-0 text-sm text-secondary", props.class)}
    >
      {props.children ?? context.label()}
    </Text>
  );
}

export function ProgressValueLabel(props: TextProps): JSX.Element {
  const context = useProgressContext();
  return (
    <Text
      {...props}
      class={mergeClasses(
        "flex-none text-sm font-mono text-muted",
        props.class,
      )}
    >
      {props.children ?? context.valueLabel() ?? "In progress"}
    </Text>
  );
}

export interface ProgressProps
  extends Omit<ProgressRootProps, "children" | "class"> {
  /** Classes applied to the visual track, preserving the original shorthand. */
  class?: string;
  size?: ProgressSize;
}

/** Compact progress bar; use ProgressRoot and parts for custom composition. */
export function Progress(props: ProgressProps): JSX.Element {
  const forwarded = omit(props, "class", "size");
  return (
    <ProgressRoot {...forwarded}>
      <ProgressTrack class={props.class} size={props.size}>
        <ProgressFill />
      </ProgressTrack>
    </ProgressRoot>
  );
}

export interface ProgressCircleProps
  extends Omit<ProgressRootProps, "children" | "class"> {
  class?: string;
  size?: ProgressSize;
}

const progressCircleSize = (size: ProgressSize | undefined) =>
  size === "xs"
    ? "w-3 h-3"
    : size === "sm"
      ? "w-4 h-4"
      : size === "lg"
        ? "w-6 h-6"
        : "w-5 h-5";

/** Compact circular progress indicator using the same semantic range contract. */
export function ProgressCircle(props: ProgressCircleProps): JSX.Element {
  const forwarded = omit(props, "class", "size");
  const details = createMemo(() =>
    normalizeProgressValue(props.value, props.minValue, props.maxValue),
  );
  const source = createMemo(() =>
    progressCircleSource(props.indeterminate ? 30 : details().percent),
  );
  const graphic = () => (
    <Svg
      source={source()}
      aria-hidden="true"
      class="absolute inset-0 w-full h-full flex-none"
    />
  );
  return (
    <ProgressRoot
      {...forwarded}
      class={mergeClasses(
        "relative flex-none items-center justify-center gap-0 text-accent",
        progressCircleSize(props.size),
        props.class,
      )}
    >
      <Show when={props.indeterminate} fallback={graphic()}>
        <Spinner decorative class="absolute inset-0 w-full h-full" />
      </Show>
    </ProgressRoot>
  );
}
