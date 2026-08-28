import type { Handle } from "@wabou/core/renderer";
import { mergeClasses } from "@wabou/core/style";
import { createSignal, type JSX, onCleanup } from "solid-js";
import type { Placement } from "../primitives";
import { Popover, Text } from "../primitives";
import type { PopupMotionProps } from "./popover";
import { componentsElevation, useComponentsTheme } from "./theme";
import { createTooltipDelayController } from "./tooltip-state";

export interface TooltipTriggerProps {
  ref(node: Handle): void;
  onPointerEnter(): void;
  onPointerLeave(): void;
  onFocus(): void;
  onBlur(): void;
  onKeyDown(event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }): void;
}

export interface TooltipProps extends PopupMotionProps {
  trigger(props: TooltipTriggerProps): JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?(open: boolean): void;
  openDelay?: number;
  closeDelay?: number;
  placement?: Placement;
  offset?: number;
  contentClass?: string;
  disabled?: boolean;
}

let tooltipId = 0;

/** A delayed, non-interactive label for pointer and keyboard focus targets. */
export function Tooltip(props: TooltipProps): JSX.Element {
  const theme = useComponentsTheme();
  const id = `wabou-tooltip-${++tooltipId}`;
  const [uncontrolledOpen, setUncontrolledOpen] = createSignal(
    props.defaultOpen ?? false,
  );
  const open = () => !props.disabled && (props.open ?? uncontrolledOpen());
  const setOpen = (next: boolean) => {
    if (props.disabled) next = false;
    if (props.open === undefined) setUncontrolledOpen(next);
    props.onOpenChange?.(next);
  };
  const delay = createTooltipDelayController({
    openDelay: () => props.openDelay ?? 500,
    closeDelay: () => props.closeDelay ?? 80,
    setOpen,
  });
  onCleanup(delay.dispose);

  return (
    <Popover
      open={open()}
      onOpenChange={(next) => !next && delay.closeNow()}
      placement={props.placement ?? "top"}
      offset={props.offset ?? 8}
      contentRole="presentation"
      popupRole="tooltip"
      outsidePointerStrategy="passthrough"
      contentInteractionBlocked
      closeOnEscape
      restoreFocus={false}
      contentClass={mergeClasses(
        "max-w-xs rounded-lg border border-subtle bg-surface px-2 py-1.5",
        props.contentClass,
      )}
      contentShadows={componentsElevation(theme(), "floating")}
      motion={props.motion}
      trigger={(popover) =>
        props.trigger({
          ref: popover.ref,
          onPointerEnter: delay.scheduleOpen,
          onPointerLeave: delay.scheduleClose,
          onFocus: delay.openNow,
          onBlur: delay.closeNow,
          onKeyDown: (event) => {
            if (event.key === "Escape") delay.closeNow();
          },
        })
      }
    >
      <Text
        id={id}
        role="tooltip"
        class="whitespace-normal text-xs text-primary"
      >
        {props.children}
      </Text>
    </Popover>
  );
}

export { createTooltipDelayController } from "./tooltip-state";
