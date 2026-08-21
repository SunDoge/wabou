import { rgba } from "@wabou/core";
import type { WabouPointerEvent } from "@wabou/core/renderer";
import type { Affine2D } from "@wabou/core/style";
import {
  createComponent,
  createContext,
  createEffect,
  createSignal,
  type JSX,
  omit,
  onCleanup,
  untrack,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import {
  type AnimationControls,
  animate,
  useReducedMotion,
} from "../animation";
import {
  createMeasuredSize,
  Modal,
  type ModalOpenChangeReason,
  type ModalProps,
  Text,
  type TextProps,
  View,
  type ViewProps,
} from "../primitives";
import { Button, type ButtonProps } from "./button";
import { join } from "./class-names";
import { componentsElevation, useComponentsTheme } from "./theme";

export type DrawerDirection = "top" | "right" | "bottom" | "left";
export type DrawerOpenChangeReason = ModalOpenChangeReason | "drag";

interface DrawerContextValue {
  direction(): DrawerDirection;
  close(): void;
  onPointerDown(event: WabouPointerEvent): void;
  onPointerMove(event: WabouPointerEvent): void;
  onPointerUp(): void;
  onPointerCancel(): void;
}

const DrawerContext = createContext<DrawerContextValue>();

function useDrawer(): DrawerContextValue {
  const context = useContext(DrawerContext);
  if (!context) throw new Error("Drawer child must be used inside Drawer");
  return context;
}

const drawerGeometry = (direction: DrawerDirection) =>
  match(direction)
    .with("left", () => ({
      backdrop: { "align-items": "stretch", "justify-content": "flex-start" },
      content: "h-full w-[420px] max-w-[80%] rounded-xl border-r",
      motion: { fromX: -48 },
    }))
    .with("right", () => ({
      backdrop: { "align-items": "stretch", "justify-content": "flex-end" },
      content: "h-full w-[420px] max-w-[80%] rounded-xl border-l",
      motion: { fromX: 48 },
    }))
    .with("top", () => ({
      backdrop: { "align-items": "flex-start", "justify-content": "stretch" },
      content: "w-full max-h-[80%] rounded-xl border-b",
      motion: { fromY: -48 },
    }))
    .with("bottom", () => ({
      backdrop: { "align-items": "flex-end", "justify-content": "stretch" },
      content: "w-full max-h-[80%] rounded-xl border-t",
      motion: { fromY: 48 },
    }))
    .exhaustive();

export function drawerDragOffset(
  direction: DrawerDirection,
  rawDelta: number,
): number {
  const outwardSign = direction === "right" || direction === "bottom" ? 1 : -1;
  return outwardSign * Math.max(0, outwardSign * rawDelta);
}

export function drawerShouldDismiss(
  offset: number,
  size: number,
  threshold: number,
): boolean {
  // ResizeObserver may not have published the first content measurement when
  // a user starts dragging during the enter transition. Keep that first drag
  // useful with a conservative physical-distance fallback.
  const dismissDistance = size > 0 ? size * threshold : 80;
  return Math.abs(offset) >= dismissDistance;
}

export interface DrawerProps
  extends Omit<
    ModalProps,
    | "defaultOpen"
    | "onOpenChange"
    | "contentClass"
    | "contentTransform"
    | "motion"
  > {
  defaultOpen?: boolean;
  onOpenChange?(open: boolean, reason: DrawerOpenChangeReason): void;
  direction?: DrawerDirection;
  dismissible?: boolean;
  dismissThreshold?: number;
  contentClass?: string;
}

/** A focus-isolated edge drawer with a captured native drag-to-dismiss gesture. */
export function Drawer(props: DrawerProps): JSX.Element {
  const theme = useComponentsTheme();
  const reducedMotion = useReducedMotion();
  const measured = createMeasuredSize();
  const [uncontrolledOpen, setUncontrolledOpen] = createSignal(
    untrack(() => props.defaultOpen ?? false),
  );
  const [dragOffset, setDragOffset] = createSignal(0, { ownedWrite: true });
  const [dragging, setDragging] = createSignal(false, { ownedWrite: true });
  const direction = () => props.direction ?? "bottom";
  const open = () => props.open ?? uncontrolledOpen();
  const axisSize = () =>
    direction() === "left" || direction() === "right"
      ? measured.width()
      : measured.height();
  let startCoordinate = 0;
  let snapControls: AnimationControls | undefined;
  const setOpen = (next: boolean, reason: DrawerOpenChangeReason) => {
    if (next) setDragOffset(0);
    if (props.open === undefined) setUncontrolledOpen(next);
    props.onOpenChange?.(next, reason);
  };
  const snapBack = () => {
    snapControls?.cancel();
    if (reducedMotion()) {
      setDragOffset(0);
      return;
    }
    snapControls = animate(dragOffset(), 0, {
      duration: 0.22,
      ease: [0.22, 1, 0.36, 1],
      onUpdate: setDragOffset,
    });
  };
  const finishDrag = (cancelled = false) => {
    if (!dragging()) return;
    setDragging(false);
    const threshold = Math.min(
      0.9,
      Math.max(0.05, props.dismissThreshold ?? 0.25),
    );
    if (
      !cancelled &&
      (props.dismissible ?? true) &&
      drawerShouldDismiss(dragOffset(), axisSize(), threshold)
    ) {
      setOpen(false, "drag");
      return;
    }
    snapBack();
  };
  const coordinate = (event: WabouPointerEvent) =>
    direction() === "left" || direction() === "right"
      ? event.clientX
      : event.clientY;
  const context: DrawerContextValue = {
    direction,
    close: () => setOpen(false, "programmatic"),
    onPointerDown(event) {
      if (event.button !== 0 || !(props.dismissible ?? true)) return;
      snapControls?.cancel();
      startCoordinate = coordinate(event);
      setDragging(true);
      event.preventDefault();
    },
    onPointerMove(event) {
      if (!dragging() || event.buttons === 0) return;
      setDragOffset(
        drawerDragOffset(direction(), coordinate(event) - startCoordinate),
      );
    },
    onPointerUp: () => finishDrag(false),
    onPointerCancel: () => finishDrag(true),
  };
  const placement = () => drawerGeometry(direction());
  const transform = (base: Affine2D): Affine2D => [
    base[0],
    base[1],
    base[2],
    base[3],
    base[4] +
      (direction() === "left" || direction() === "right" ? dragOffset() : 0),
    base[5] +
      (direction() === "top" || direction() === "bottom" ? dragOffset() : 0),
  ];
  createEffect(open, (isOpen) => {
    if (isOpen) setDragOffset(0);
  });
  onCleanup(() => snapControls?.cancel());
  return createComponent(DrawerContext, {
    value: context,
    get children() {
      return (
        <Modal
          {...props}
          open={open()}
          onOpenChange={(next, reason) => setOpen(next, reason)}
          motion={{ duration: 0.22, ...placement().motion }}
          backdropStyle={{
            "background-color": rgba(0x00000066),
            ...placement().backdrop,
            ...props.backdropStyle,
          }}
          contentRef={(node) => {
            measured.ref(node);
            props.contentRef?.(node);
          }}
          contentClass={join(
            "relative min-w-0 min-h-0 flex flex-col border-subtle bg-surface",
            placement().content,
            props.contentClass,
          )}
          contentTransform={transform}
          contentShadows={
            props.contentShadows === undefined
              ? componentsElevation(theme(), "modal")
              : props.contentShadows
          }
        >
          {typeof props.children === "function"
            ? props.children({ close: context.close })
            : props.children}
        </Modal>
      );
    },
  });
}

export function DrawerHandle(props: ViewProps): JSX.Element {
  const drawer = useDrawer();
  const forwarded = omit(
    props,
    "class",
    "onPointerDown",
    "onPointerMove",
    "onPointerUp",
    "onPointerCancel",
    "onClick",
  );
  const hitArea = () =>
    match(drawer.direction())
      .with("left", () => "absolute right-0 top-0 w-8 h-full")
      .with("right", () => "absolute left-0 top-0 w-8 h-full")
      .with("top", "bottom", () => "w-full h-8")
      .exhaustive();
  const indicator = () =>
    match(drawer.direction())
      .with("left", "right", () => "w-1.5 h-20")
      .with("top", "bottom", () => "w-20 h-1.5")
      .exhaustive();
  return (
    <View
      {...forwarded}
      role="button"
      aria-label={props["aria-label"] ?? "Drag or click to close drawer"}
      class={join(
        "flex flex-none items-center justify-center",
        hitArea(),
        props.class,
      )}
      onPointerDown={(event) => {
        drawer.onPointerDown(event);
        props.onPointerDown?.(event);
      }}
      onPointerMove={(event) => {
        drawer.onPointerMove(event);
        props.onPointerMove?.(event);
      }}
      onPointerUp={(event) => {
        drawer.onPointerUp();
        props.onPointerUp?.(event);
      }}
      onPointerCancel={(event) => {
        drawer.onPointerCancel();
        props.onPointerCancel?.(event);
      }}
      onClick={(event) => {
        drawer.close();
        props.onClick?.(event);
      }}
    >
      <View
        aria-hidden="true"
        class={join("pointer-events-none rounded-full bg-strong", indicator())}
      />
    </View>
  );
}

export function DrawerHeader(props: ViewProps): JSX.Element {
  return (
    <View {...props} class={join("flex flex-col gap-1 p-5", props.class)} />
  );
}

export function DrawerFooter(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={join("mt-auto flex flex-col gap-2 p-5", props.class)}
    />
  );
}

export function DrawerTitle(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={join("text-lg font-semibold text-primary", props.class)}
    />
  );
}

export function DrawerDescription(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={join("whitespace-normal text-sm text-muted", props.class)}
    />
  );
}

export function DrawerClose(props: ButtonProps): JSX.Element {
  const drawer = useDrawer();
  const forwarded = omit(props, "onClick");
  return (
    <Button
      {...forwarded}
      onClick={(event) => {
        drawer.close();
        props.onClick?.(event);
      }}
    />
  );
}
