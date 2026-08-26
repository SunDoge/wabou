import type { WabouPointerEvent } from "@wabou/core/renderer";
import arrowDown from "lucide-static/icons/arrow-down.svg?raw";
import arrowLeft from "lucide-static/icons/arrow-left.svg?raw";
import arrowRight from "lucide-static/icons/arrow-right.svg?raw";
import arrowUp from "lucide-static/icons/arrow-up.svg?raw";
import {
  createComponent,
  createContext,
  createEffect,
  createSignal,
  createUniqueId,
  type JSX,
  omit,
  onCleanup,
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
  Icon,
  translate2d,
  View,
  type ViewProps,
} from "../primitives";
import { createControllableState } from "../primitives/interactions";
import { Button, type ButtonProps } from "./button";
import { mergeClasses } from "@wabou/core/style";

export type CarouselOrientation = "horizontal" | "vertical";

export interface CarouselApi {
  selectedIndex(): number;
  itemCount(): number;
  canScrollPrevious(): boolean;
  canScrollNext(): boolean;
  scrollPrevious(): void;
  scrollNext(): void;
  scrollTo(index: number): void;
}

interface CarouselContextValue extends CarouselApi {
  orientation(): CarouselOrientation;
  register(id: string): () => void;
}

const CarouselContext = createContext<CarouselContextValue>();

function useCarousel(): CarouselContextValue {
  const context = useContext(CarouselContext);
  if (!context) throw new Error("Carousel child must be used inside Carousel");
  return context;
}

export function normalizeCarouselIndex(
  index: number,
  count: number,
  loop: boolean,
): number {
  if (count <= 0) return 0;
  if (loop) return ((Math.trunc(index) % count) + count) % count;
  return Math.min(count - 1, Math.max(0, Math.trunc(index)));
}

export interface CarouselProps {
  "aria-label": string;
  index?: number;
  defaultIndex?: number;
  onIndexChange?(index: number): void;
  orientation?: CarouselOrientation;
  loop?: boolean;
  setApi?(api: CarouselApi): void;
  class?: string;
  children?: JSX.Element;
}

/** A native snapping carousel with captured pointer dragging and keyboard navigation. */
export function Carousel(props: CarouselProps): JSX.Element {
  const items: string[] = [];
  const [revision, setRevision] = createSignal(0, { ownedWrite: true });
  const state = createControllableState({
    value: () => props.index,
    defaultValue: normalizeCarouselIndex(props.defaultIndex ?? 0, 1, false),
    onChange: props.onIndexChange,
  });
  const count = () => {
    revision();
    return items.length;
  };
  const selectedIndex = () =>
    normalizeCarouselIndex(state.value(), count(), props.loop ?? false);
  const scrollTo = (index: number) => {
    if (count() === 0) return;
    state.set(normalizeCarouselIndex(index, count(), props.loop ?? false));
  };
  const context: CarouselContextValue = {
    orientation: () => props.orientation ?? "horizontal",
    selectedIndex,
    itemCount: count,
    canScrollPrevious: () =>
      count() > 1 && ((props.loop ?? false) || selectedIndex() > 0),
    canScrollNext: () =>
      count() > 1 && ((props.loop ?? false) || selectedIndex() < count() - 1),
    scrollPrevious: () => scrollTo(selectedIndex() - 1),
    scrollNext: () => scrollTo(selectedIndex() + 1),
    scrollTo,
    register(id) {
      items.push(id);
      setRevision((value) => value + 1);
      return () => {
        const index = items.indexOf(id);
        if (index >= 0) items.splice(index, 1);
        setRevision((value) => value + 1);
      };
    },
  };
  props.setApi?.(context);
  return createComponent(CarouselContext, {
    value: context,
    get children() {
      return (
        <View
          role="group"
          aria-label={props["aria-label"]}
          class={mergeClasses("relative min-w-0 min-h-0", props.class)}
          onKeyDown={(event) => {
            const handled = match({
              orientation: context.orientation(),
              key: event.key,
            })
              .with({ orientation: "horizontal", key: "ArrowLeft" }, () => {
                context.scrollPrevious();
                return true;
              })
              .with({ orientation: "horizontal", key: "ArrowRight" }, () => {
                context.scrollNext();
                return true;
              })
              .with({ orientation: "vertical", key: "ArrowUp" }, () => {
                context.scrollPrevious();
                return true;
              })
              .with({ orientation: "vertical", key: "ArrowDown" }, () => {
                context.scrollNext();
                return true;
              })
              .otherwise(() => false);
            if (handled) event.preventDefault();
          }}
        >
          {props.children}
        </View>
      );
    },
  });
}

export interface CarouselContentProps extends Omit<ViewProps, "transform"> {
  trackClass?: string;
  dragThreshold?: number;
}

export function CarouselContent(props: CarouselContentProps): JSX.Element {
  const carousel = useCarousel();
  const measured = createMeasuredSize();
  const reducedMotion = useReducedMotion();
  const [offset, setOffset] = createSignal(0, { ownedWrite: true });
  const [dragDelta, setDragDelta] = createSignal(0, { ownedWrite: true });
  const [dragging, setDragging] = createSignal(false, { ownedWrite: true });
  let startCoordinate = 0;
  let controls: AnimationControls | undefined;
  const axisSize = () =>
    carousel.orientation() === "horizontal"
      ? measured.width()
      : measured.height();
  const targetOffset = () => -carousel.selectedIndex() * axisSize();
  const coordinate = (event: WabouPointerEvent) =>
    carousel.orientation() === "horizontal" ? event.clientX : event.clientY;
  const settle = () => {
    controls?.cancel();
    const target = targetOffset();
    if (reducedMotion() || Math.abs(target - offset()) < 0.5) {
      setOffset(target);
      return;
    }
    controls = animate(offset(), target, {
      duration: 0.28,
      ease: [0.22, 1, 0.36, 1],
      onUpdate: setOffset,
    });
  };
  createEffect(
    () => [targetOffset(), measured.measured()] as const,
    () => {
      if (!dragging()) settle();
    },
  );
  onCleanup(() => controls?.cancel());
  const stopDragging = (cancelled = false) => {
    if (!dragging()) return;
    const delta = dragDelta();
    const threshold = Math.max(12, axisSize() * (props.dragThreshold ?? 0.15));
    setOffset(offset() + delta);
    setDragDelta(0);
    setDragging(false);
    if (!cancelled && Math.abs(delta) >= threshold) {
      if (delta < 0) carousel.scrollNext();
      else carousel.scrollPrevious();
    }
    requestAnimationFrame(settle);
  };
  const forwarded = omit(
    props,
    "trackClass",
    "dragThreshold",
    "children",
    "class",
    "ref",
    "onPointerDown",
    "onPointerMove",
    "onPointerUp",
    "onPointerCancel",
  );
  return (
    <View
      {...forwarded}
      ref={(node) => {
        measured.ref(node);
        props.ref?.(node);
      }}
      class={mergeClasses("min-w-0 min-h-0 overflow-hidden", props.class)}
      onPointerDown={(event) => {
        if (event.button !== 0 || axisSize() <= 0) return;
        controls?.cancel();
        startCoordinate = coordinate(event);
        setDragDelta(0);
        setDragging(true);
        event.preventDefault();
        props.onPointerDown?.(event);
      }}
      onPointerMove={(event) => {
        if (dragging() && event.buttons !== 0)
          setDragDelta(coordinate(event) - startCoordinate);
        props.onPointerMove?.(event);
      }}
      onPointerUp={(event) => {
        stopDragging();
        props.onPointerUp?.(event);
      }}
      onPointerCancel={(event) => {
        stopDragging(true);
        props.onPointerCancel?.(event);
      }}
    >
      <View
        class={mergeClasses(
          "flex w-full h-full flex-none",
          carousel.orientation() === "horizontal" ? "flex-row" : "flex-col",
          props.trackClass,
        )}
        transform={
          carousel.orientation() === "horizontal"
            ? translate2d(offset() + dragDelta(), 0)
            : translate2d(0, offset() + dragDelta())
        }
      >
        {props.children}
      </View>
    </View>
  );
}

export interface CarouselItemProps extends ViewProps {
  "aria-label"?: string;
}

export function CarouselItem(props: CarouselItemProps): JSX.Element {
  const carousel = useCarousel();
  const id = createUniqueId();
  const unregister = carousel.register(id);
  onCleanup(unregister);
  return (
    <View
      {...props}
      role="group"
      aria-label={props["aria-label"]}
      class={mergeClasses(
        "min-w-0 min-h-0 flex-none",
        carousel.orientation() === "horizontal" ? "w-full" : "h-full",
        props.class,
      )}
    />
  );
}

function CarouselNavigationButton(
  props: ButtonProps & { direction: "previous" | "next" },
): JSX.Element {
  const carousel = useCarousel();
  const previous = props.direction === "previous";
  const forwarded = omit(props, "direction", "children", "onClick");
  return (
    <Button
      {...forwarded}
      aria-label={
        props["aria-label"] ?? (previous ? "Previous slide" : "Next slide")
      }
      variant={props.variant ?? "outline"}
      size={props.size ?? "icon"}
      disabled={
        props.disabled ??
        (previous ? !carousel.canScrollPrevious() : !carousel.canScrollNext())
      }
      onClick={(event) => {
        if (previous) carousel.scrollPrevious();
        else carousel.scrollNext();
        props.onClick?.(event);
      }}
    >
      {props.children ?? (
        <Icon
          aria-hidden="true"
          source={match({
            previous,
            orientation: carousel.orientation(),
          })
            .with({ orientation: "vertical", previous: true }, () => arrowUp)
            .with({ orientation: "vertical", previous: false }, () => arrowDown)
            .with({ previous: true }, () => arrowLeft)
            .otherwise(() => arrowRight)}
          size={16}
        />
      )}
    </Button>
  );
}

export function CarouselPrevious(
  props: Omit<ButtonProps, "children"> & { children?: JSX.Element },
): JSX.Element {
  return <CarouselNavigationButton {...props} direction="previous" />;
}

export function CarouselNext(
  props: Omit<ButtonProps, "children"> & { children?: JSX.Element },
): JSX.Element {
  return <CarouselNavigationButton {...props} direction="next" />;
}
