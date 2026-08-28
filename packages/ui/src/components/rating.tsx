import type { Handle } from "@wabou/core/renderer";
import { scale2d } from "@wabou/core/style";
import star from "lucide-static/icons/star.svg?raw";
import { createSignal, For, type JSX, onCleanup } from "solid-js";
import { match, P } from "ts-pattern";
import { createTransition, useReducedMotion } from "../animation";
import { Button as HeadlessButton, Icon, View } from "../primitives";
import {
  createControllableState,
  createRovingFocus,
} from "../primitives/interactions";
import { mergeClasses } from "@wabou/core/style";
import {
  clampRatingValue,
  normalizeRatingMax,
  ratingLabel,
} from "./rating-state";

export {
  clampRatingValue,
  normalizeRatingMax,
  ratingLabel,
} from "./rating-state";

export interface RatingProps {
  value?: number;
  defaultValue?: number;
  max?: number;
  disabled?: boolean;
  readOnly?: boolean;
  allowClear?: boolean;
  label: string;
  class?: string;
  size?: number;
  onValueChange?: (value: number) => void;
}

function RatingIcon(props: {
  highlighted: boolean;
  size: number;
}): JSX.Element {
  const reducedMotion = useReducedMotion();
  const emphasis = createTransition(() => (props.highlighted ? 1 : 0), {
    duration: 0.12,
    ease: "easeOut",
    reducedMotion,
  });
  return (
    <Icon
      aria-hidden="true"
      source={star}
      size={props.size}
      fill={props.highlighted ? "currentColor" : "none"}
      class={props.highlighted ? "text-accent" : "text-muted"}
      transform={scale2d(0.9 + emphasis.value() * 0.1)}
    />
  );
}

export function Rating(props: RatingProps): JSX.Element {
  const max = () => normalizeRatingMax(props.max);
  const normalize = (value: number | undefined) =>
    clampRatingValue(value, max());
  const state = createControllableState({
    value: () =>
      props.value === undefined ? undefined : normalize(props.value),
    defaultValue: normalize(props.defaultValue),
    disabled: () => (props.disabled ?? false) || (props.readOnly ?? false),
    onChange: props.onValueChange,
  });
  const value = () => normalize(state.value());
  const [preview, setPreview] = createSignal<number>();
  let previewGeneration = 0;
  const previewItem = (item: number) => {
    previewGeneration++;
    setPreview(item);
  };
  const clearPreviewAfterPointerDispatch = () => {
    const generation = ++previewGeneration;
    queueMicrotask(() => {
      if (generation === previewGeneration) setPreview(undefined);
    });
  };
  const shownValue = () => preview() ?? value();
  const disabled = () => props.disabled ?? false;
  const inert = () => disabled() || (props.readOnly ?? false);
  const items = () => Array.from({ length: max() }, (_, index) => index + 1);
  const select = (next: number) => {
    if (inert()) return;
    const normalized = normalize(next);
    state.set(props.allowClear && normalized === value() ? 0 : normalized);
  };
  const roving = createRovingFocus({
    orientation: () => "horizontal",
    onMove: (id) => select(Number(id)),
  });

  return (
    <View
      role="radiogroup"
      aria-label={props.label}
      aria-disabled={disabled() || undefined}
      aria-orientation="horizontal"
      class={mergeClasses("flex flex-col items-start gap-1.5", props.class)}
      style={{ opacity: disabled() ? 0.45 : 1 }}
    >
      <View class="flex flex-row items-center gap-0.5">
        <For each={items()}>
          {(item) => {
            const checked = () => value() === item;
            const highlighted = () => item <= shownValue();
            let unregister: (() => void) | undefined;
            onCleanup(() => unregister?.());
            return (
              <HeadlessButton
                unstyled
                role="radio"
                aria-label={ratingLabel(item)}
                aria-checked={checked()}
                selected={checked()}
                disabled={disabled()}
                focusOrder={
                  disabled()
                    ? -1
                    : checked() || (value() === 0 && item === 1)
                      ? 0
                      : -1
                }
                ref={(node: Handle) => {
                  unregister?.();
                  unregister = roving.register({
                    id: String(item),
                    target: node,
                    disabled: inert,
                  });
                }}
                class={(buttonState) =>
                  mergeClasses(
                    "w-8 h-8 items-center justify-center rounded-md border border-transparent",
                    match({
                      focused: buttonState.focusVisible,
                      hovered: buttonState.hovered,
                    })
                      .with({ focused: true }, () => "border-focus bg-control")
                      .with({ hovered: true }, () => "bg-control-hover")
                      .with(
                        { focused: false, hovered: false },
                        () => "bg-transparent",
                      )
                      .exhaustive(),
                  )
                }
                onPointerEnter={() => !inert() && previewItem(item)}
                onPointerLeave={clearPreviewAfterPointerDispatch}
                onClick={() => select(item)}
                onKeyDown={(event) => {
                  const handled = match(event.key)
                    .with(
                      P.union(
                        "Home",
                        "End",
                        "ArrowLeft",
                        "ArrowRight",
                        "ArrowUp",
                        "ArrowDown",
                      ),
                      () => roving.move(String(item), event.key),
                    )
                    .otherwise(() => false);
                  if (handled) event.preventDefault();
                }}
              >
                <RatingIcon
                  highlighted={highlighted()}
                  size={props.size ?? 20}
                />
              </HeadlessButton>
            );
          }}
        </For>
      </View>
    </View>
  );
}
