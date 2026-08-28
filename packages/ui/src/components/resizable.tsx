import type { WabouPointerEvent } from "@wabou/core/renderer";
import {
  createContext,
  createMemo,
  createSignal,
  type JSX,
  useContext,
} from "solid-js";
import {
  createMeasuredSize,
  View,
  type ViewProps,
  type WabouStyle,
} from "../primitives";
import { mergeClasses } from "@wabou/core/style";
import { createControllableState } from "./state";

export type ResizableDirection = "horizontal" | "vertical";

export interface ResizablePanelDefinition {
  id: string;
  defaultSize: number;
  minSize?: number;
  maxSize?: number;
}

export type ResizablePanelSizes = Readonly<Record<string, number>>;

function finitePercentage(value: number, name: string): number {
  if (!Number.isFinite(value) || value < 0 || value > 100) {
    throw new RangeError(`${name} must be a finite percentage from 0 to 100`);
  }
  return value;
}

export function validateResizableSizes(
  panels: readonly ResizablePanelDefinition[],
  sizes?: ResizablePanelSizes,
): ResizablePanelSizes {
  if (panels.length < 2) {
    throw new RangeError("resizable panels require at least two definitions");
  }
  const ids = new Set<string>();
  const candidates = panels.map((panel) => {
    if (!panel.id || ids.has(panel.id)) {
      throw new Error(`resizable panel id must be unique: ${panel.id}`);
    }
    ids.add(panel.id);
    const min = finitePercentage(panel.minSize ?? 0, `${panel.id}.minSize`);
    const max = finitePercentage(panel.maxSize ?? 100, `${panel.id}.maxSize`);
    if (min > max) throw new RangeError(`${panel.id}.minSize exceeds maxSize`);
    const candidate = finitePercentage(
      sizes?.[panel.id] ?? panel.defaultSize,
      `${panel.id}.size`,
    );
    if (candidate < min || candidate > max) {
      throw new RangeError(`${panel.id}.size is outside its min/max range`);
    }
    return candidate;
  });
  const total = candidates.reduce((sum, value) => sum + value, 0);
  if (Math.abs(total - 100) > 0.001) {
    throw new RangeError(
      `resizable panel sizes must total 100; received ${total}`,
    );
  }
  return Object.fromEntries(
    panels.map((panel, index) => [panel.id, candidates[index]]),
  );
}

export interface ResizablePanelState {
  readonly panels: readonly ResizablePanelDefinition[];
  sizes(): ResizablePanelSizes;
  size(id: string): number;
  pairRange(before: string, after: string): { min: number; max: number };
  resizePair(before: string, after: string, beforeSize: number): boolean;
  resetPair(before: string, after: string): boolean;
}

export function createResizablePanelState(options: {
  panels: readonly ResizablePanelDefinition[];
  value?: () => ResizablePanelSizes | undefined;
  defaultValue?: ResizablePanelSizes;
  onValueChange?: (sizes: ResizablePanelSizes) => void;
}): ResizablePanelState {
  const defaults = validateResizableSizes(options.panels, options.defaultValue);
  const definitions = new Map(options.panels.map((panel) => [panel.id, panel]));
  const state = createControllableState<ResizablePanelSizes>({
    value: () => {
      const value = options.value?.();
      return value === undefined
        ? undefined
        : validateResizableSizes(options.panels, value);
    },
    defaultValue: defaults,
    onChange: options.onValueChange,
  });
  const requirePanel = (id: string) => {
    const panel = definitions.get(id);
    if (!panel) throw new Error(`unknown resizable panel: ${id}`);
    return panel;
  };
  const size = (id: string) => {
    requirePanel(id);
    const value = state.value()[id];
    if (!Number.isFinite(value))
      throw new Error(`missing resizable panel size: ${id}`);
    return value;
  };
  const pairRange = (before: string, after: string) => {
    const beforePanel = requirePanel(before);
    const afterPanel = requirePanel(after);
    if (before === after)
      throw new Error("resizable handle requires two panels");
    const pair = size(before) + size(after);
    return {
      min: Math.max(
        beforePanel.minSize ?? 0,
        pair - (afterPanel.maxSize ?? 100),
      ),
      max: Math.min(
        beforePanel.maxSize ?? 100,
        pair - (afterPanel.minSize ?? 0),
      ),
    };
  };
  const resizePair = (before: string, after: string, beforeSize: number) => {
    const range = pairRange(before, after);
    const pair = size(before) + size(after);
    const nextBefore = Math.max(range.min, Math.min(range.max, beforeSize));
    const nextAfter = pair - nextBefore;
    const current = state.value();
    if (current[before] === nextBefore && current[after] === nextAfter)
      return false;
    return state.set({ ...current, [before]: nextBefore, [after]: nextAfter });
  };
  return {
    panels: options.panels,
    sizes: state.value,
    size,
    pairRange,
    resizePair,
    resetPair(before, after) {
      const beforeDefault = defaults[before];
      const afterDefault = defaults[after];
      if (beforeDefault === undefined || afterDefault === undefined) {
        pairRange(before, after);
        return false;
      }
      const pair = size(before) + size(after);
      return resizePair(
        before,
        after,
        pair * (beforeDefault / (beforeDefault + afterDefault)),
      );
    },
  };
}

interface ResizableContextValue {
  direction: () => ResizableDirection;
  state: ResizablePanelState;
  axisSize(): number;
}

const ResizableContext = createContext<ResizableContextValue>();

function useResizable(): ResizableContextValue {
  const context = useContext(ResizableContext);
  if (!context)
    throw new Error("Resizable parts must be inside ResizablePanelGroup");
  return context;
}

export interface ResizablePanelGroupProps {
  panels: readonly ResizablePanelDefinition[];
  children?: JSX.Element;
  direction?: ResizableDirection;
  value?: ResizablePanelSizes;
  defaultValue?: ResizablePanelSizes;
  onValueChange?: (sizes: ResizablePanelSizes) => void;
  "aria-label": string;
  class?: string;
}

export function ResizablePanelGroup(
  props: ResizablePanelGroupProps,
): JSX.Element {
  let measuredWidth = 0;
  let measuredHeight = 0;
  const measured = createMeasuredSize({
    onChange(size) {
      measuredWidth = size.width;
      measuredHeight = size.height;
    },
  });
  const state = createResizablePanelState({
    panels: props.panels,
    value: () => props.value,
    defaultValue: props.defaultValue,
    onValueChange: props.onValueChange,
  });
  const direction = () => props.direction ?? "horizontal";
  const context: ResizableContextValue = {
    direction,
    state,
    axisSize: () =>
      direction() === "horizontal" ? measuredWidth : measuredHeight,
  };
  return (
    <ResizableContext value={context}>
      <View
        ref={measured.ref}
        role="group"
        aria-label={props["aria-label"]}
        class={mergeClasses(
          "w-full h-full min-w-0 min-h-0 flex overflow-hidden",
          direction() === "horizontal" ? "flex-row" : "flex-col",
          props.class,
        )}
      >
        {props.children}
      </View>
    </ResizableContext>
  );
}

export function ResizablePanel(props: {
  id: string;
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  const context = useResizable();
  const style = createMemo<WabouStyle>(() =>
    context.direction() === "horizontal"
      ? { width: `${context.state.size(props.id)}%` }
      : { height: `${context.state.size(props.id)}%` },
  );
  return (
    <View
      role="group"
      aria-label={props.id}
      class={mergeClasses(
        "min-w-0 min-h-0 flex-none overflow-hidden",
        props.class,
      )}
      style={style()}
    >
      {props.children}
    </View>
  );
}

export interface ResizableHandleProps {
  before: string;
  after: string;
  "aria-label": string;
  keyboardStep?: number;
  class?: string;
}

export function ResizableHandle(props: ResizableHandleProps): JSX.Element {
  const context = useResizable();
  const [dragging, setDragging] = createSignal(false);
  const [hovered, setHovered] = createSignal(false);
  let startCoordinate = 0;
  let startSize = 0;
  const coordinate = (event: WabouPointerEvent) =>
    context.direction() === "horizontal" ? event.clientX : event.clientY;
  const range = () => context.state.pairRange(props.before, props.after);
  const moveTo = (value: number) =>
    context.state.resizePair(props.before, props.after, value);
  const onPointerDown = (event: WabouPointerEvent) => {
    if (event.button !== 0) return;
    event.preventDefault();
    startCoordinate = coordinate(event);
    startSize = context.state.size(props.before);
    setDragging(true);
  };
  const onPointerMove = (event: WabouPointerEvent) => {
    if (!dragging() || event.buttons === 0) return;
    const axisSize = context.axisSize();
    if (axisSize <= 0) return;
    moveTo(
      startSize + ((coordinate(event) - startCoordinate) / axisSize) * 100,
    );
  };
  const stopDragging = () => setDragging(false);
  const onKeyDown: NonNullable<ViewProps["onKeyDown"]> = (event) => {
    const step = Math.max(0.1, props.keyboardStep ?? 2);
    const current = context.state.size(props.before);
    const next =
      event.key === "Home"
        ? range().min
        : event.key === "End"
          ? range().max
          : event.key ===
              (context.direction() === "horizontal" ? "ArrowLeft" : "ArrowUp")
            ? current - step
            : event.key ===
                (context.direction() === "horizontal"
                  ? "ArrowRight"
                  : "ArrowDown")
              ? current + step
              : undefined;
    if (next === undefined) return;
    event.preventDefault();
    moveTo(next);
  };
  return (
    <View
      role="separator"
      aria-label={props["aria-label"]}
      aria-valuemin={range().min}
      aria-valuemax={range().max}
      aria-valuenow={context.state.size(props.before)}
      aria-valuetext={`${Math.round(context.state.size(props.before))} percent`}
      focusOrder={0}
      class={mergeClasses(
        "flex-none rounded-sm",
        context.direction() === "horizontal" ? "w-2 h-full" : "w-full h-2",
        dragging() || hovered() ? "bg-accent" : "bg-control",
        props.class,
      )}
      onPointerEnter={() => setHovered(true)}
      onPointerLeave={() => setHovered(false)}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={stopDragging}
      onPointerCancel={stopDragging}
      onDblClick={() => context.state.resetPair(props.before, props.after)}
      onKeyDown={onKeyDown}
    />
  );
}
