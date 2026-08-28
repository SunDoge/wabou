import type { Handle } from "@wabou/core/renderer";
import type { Shadow } from "@wabou/core/style";
import { mergeClasses } from "@wabou/core/style";
import { type JSX, onCleanup } from "solid-js";
import type { Placement } from "../primitives";
import { Popover as HeadlessPopover } from "../primitives";
import { createDelayedOpenController } from "./delayed-open";
import type { PopupMotionProps } from "./popover";
import { createControllableState } from "./state";
import { componentsElevation, useComponentsTheme } from "./theme";

export interface HoverCardTriggerProps {
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
  "aria-haspopup": "dialog";
  "aria-expanded": boolean;
}

export interface HoverCardProps extends PopupMotionProps {
  "aria-label": string;
  trigger(props: HoverCardTriggerProps): JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?(open: boolean): void;
  openDelay?: number;
  closeDelay?: number;
  placement?: Placement;
  offset?: number;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  disabled?: boolean;
}

/** A preview surface that tolerates pointer travel between trigger and card. */
export function HoverCard(props: HoverCardProps): JSX.Element {
  const theme = useComponentsTheme();
  const state = createControllableState({
    value: () => props.open,
    defaultValue: props.defaultOpen ?? false,
    disabled: () => props.disabled ?? false,
    onChange: props.onOpenChange,
  });
  const open = () => !(props.disabled ?? false) && state.value();
  const delay = createDelayedOpenController({
    openDelay: () => Math.max(0, props.openDelay ?? 400),
    closeDelay: () => Math.max(0, props.closeDelay ?? 200),
    setOpen: (next) => state.set(next),
  });
  onCleanup(delay.dispose);

  return (
    <HeadlessPopover
      open={open()}
      onOpenChange={(next) => {
        if (!next) delay.closeNow();
      }}
      aria-label={props["aria-label"]}
      placement={props.placement ?? "bottom-start"}
      offset={props.offset ?? 8}
      closeOnEscape
      restoreFocus={false}
      contentClass={mergeClasses(
        "min-w-56 max-w-sm min-h-0 p-4 flex flex-col gap-3 rounded-xl border border-subtle bg-surface",
        props.contentClass,
      )}
      contentShadows={
        props.contentShadows === undefined
          ? componentsElevation(theme(), "floating")
          : props.contentShadows
      }
      motion={props.motion}
      onContentPointerEnter={delay.openNow}
      onContentPointerLeave={delay.scheduleClose}
      onContentFocusIn={delay.openNow}
      onContentFocusOut={delay.scheduleClose}
      trigger={(popover) => {
        const trigger: HoverCardTriggerProps = {
          ref: popover.ref,
          onPointerEnter: delay.scheduleOpen,
          onPointerLeave: delay.scheduleClose,
          onFocus: delay.openNow,
          onBlur: delay.scheduleClose,
          onKeyDown: (event) => {
            if (event.key === "Escape") delay.closeNow();
          },
          "aria-haspopup": "dialog",
          get "aria-expanded"() {
            return open();
          },
        };
        return props.trigger(trigger);
      }}
    >
      {props.children}
    </HeadlessPopover>
  );
}

export {
  createDelayedOpenController,
  type DelayedOpenController,
  type DelayedOpenOptions,
} from "./delayed-open";
