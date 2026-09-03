import type { Handle } from "@wabou/core/renderer";
import { mergeClasses } from "@wabou/core/style";
import { createSignal, type JSX } from "solid-js";
import { match } from "ts-pattern";
import { useReducedMotion } from "../animation";
import {
  type ButtonState,
  Button as HeadlessButton,
  translate2d,
  View,
} from "../primitives";
import { Label } from "./label";
import { componentsDisabledInteractiveClass } from "./theme";

export interface SwitchProps {
  checked?: boolean;
  defaultChecked?: boolean;
  disabled?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  label?: string;
  /** Place the label before or after the control in reading order. */
  labelPlacement?: "start" | "end";
  "aria-label"?: string;
  class?: string;
  size?: "sm" | "default";
}

function switchColors(checked: boolean, state: ButtonState): string {
  if (state.disabled) return checked ? "bg-accent" : "bg-control";
  return match({ checked, pressed: state.pressed, hovered: state.hovered })
    .with({ checked: true, pressed: true }, () => "bg-accent-pressed")
    .with({ checked: true, hovered: true }, () => "bg-accent-hover")
    .with({ checked: true }, () => "bg-accent")
    .with({ checked: false, pressed: true }, () => "bg-control-pressed")
    .with({ checked: false, hovered: true }, () => "bg-control-hover")
    .with({ checked: false }, () => "bg-control")
    .exhaustive();
}

export function switchGeometry(size: "sm" | "default") {
  return size === "sm"
    ? {
        trackClass: "w-7 h-4 p-0.5",
        thumbClass: "w-3 h-3",
        travel: 12,
      }
    : {
        trackClass: "w-9 h-5 p-0.5",
        thumbClass: "w-4 h-4",
        travel: 16,
      };
}

export function switchTrackClass(
  checked: boolean,
  state: ButtonState,
  size: "sm" | "default" = "default",
): string {
  return mergeClasses(
    "flex-none overflow-hidden rounded-full",
    switchGeometry(size).trackClass,
    switchColors(checked, state),
  );
}

export function switchControlClass(state: ButtonState): string {
  return mergeClasses(
    "w-10 h-6 p-0 flex-none items-center justify-center rounded-full border border-transparent bg-transparent",
    state.focusVisible && "border-focus",
  );
}

export function Switch(props: SwitchProps): JSX.Element {
  const [local, setLocal] = createSignal(props.defaultChecked ?? false);
  const checked = () => props.checked ?? local();
  const reducedMotion = useReducedMotion();
  const size = () => props.size ?? "default";
  const geometry = () => switchGeometry(size());
  let control: Handle | undefined;
  const toggle = () => {
    if (props.disabled) return;
    const next = !checked();
    if (props.checked === undefined) setLocal(next);
    props.onCheckedChange?.(next);
  };
  return (
    <View
      class={mergeClasses(
        "w-full min-w-0 flex items-start gap-2",
        props.labelPlacement === "start" && "flex-row-reverse",
        props.class,
      )}
    >
      <HeadlessButton
        ref={(node) => {
          control = node;
        }}
        unstyled
        role="switch"
        disabled={props.disabled}
        aria-label={props["aria-label"] ?? props.label}
        aria-checked={checked()}
        class={(state) =>
          mergeClasses(
            switchControlClass(state),
            componentsDisabledInteractiveClass(state.disabled),
          )
        }
        onClick={toggle}
        renderContent={(buttonState) => (
          <View
            aria-hidden="true"
            class={switchTrackClass(checked(), buttonState, size())}
          >
            <View
              class={mergeClasses(
                geometry().thumbClass,
                "rounded-full bg-surface",
              )}
              transform={translate2d(checked() ? geometry().travel : 0, 0)}
              nativeSpring={{
                response: reducedMotion() ? 0 : 0.16,
                damping: 1,
                epsilon: 0.02,
                targetTransform: translate2d(
                  checked() ? geometry().travel : 0,
                  0,
                ),
              }}
            />
          </View>
        )}
      />
      {props.label && (
        <Label
          control={() => control}
          disabled={props.disabled}
          class="min-w-0 flex-1 select-none whitespace-normal text-sm font-normal text-primary"
          onClick={(event) => {
            if (props.disabled || event.defaultPrevented) return;
            toggle();
          }}
        >
          {props.label}
        </Label>
      )}
    </View>
  );
}
