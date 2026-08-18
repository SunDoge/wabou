import {
  createComponent,
  createContext,
  createEffect,
  createSignal,
  type JSX,
  onCleanup,
  useContext,
} from "solid-js";

export type OverlayPlane = "floating" | "modal";

export interface DismissEvent {
  preventDefault(): void;
  stopPropagation(): void;
}

export interface DismissKeyEvent extends DismissEvent {
  key: string;
}

export type OverlayDismissReason = "escape" | "outside";

export interface OverlayLayerOptions {
  open: () => boolean;
  /** Must match the native Portal plane so dismissal follows paint order. */
  plane?: () => OverlayPlane;
  onDismiss: (reason: OverlayDismissReason) => void;
  closeOnEscape?: () => boolean;
  closeOnOutside?: () => boolean;
  restoreFocus?: () => boolean;
  returnFocus?: () => { focus(): void } | undefined;
}

export interface OverlayLayer {
  plane(): OverlayPlane;
  /** Stable native sibling order for the current open lifetime. */
  zIndex(): number;
  isTopmost(): boolean;
  onEscape(event: DismissKeyEvent): void;
  onOutside(event: DismissEvent): void;
}

// A QuickJS window owns one module graph, so this stack is naturally scoped to
// one native window. Tokens follow open order rather than component mount order:
// reopening an older overlay correctly promotes it above its siblings.
interface OpenLayer {
  token: symbol;
  plane: OverlayPlane;
  order: number;
}

const openLayers: OpenLayer[] = [];
let nextOrder = 1;

const OverlayPlaneContext = createContext<OverlayPlane>("floating");

export interface OverlayPlaneProviderProps {
  plane: OverlayPlane;
  children?: JSX.Element;
}

/** Make nested portals inherit the current native stacking plane. */
export function OverlayPlaneProvider(
  props: OverlayPlaneProviderProps,
): JSX.Element {
  return createComponent(OverlayPlaneContext, {
    get value() {
      return props.plane;
    },
    get children() {
      return props.children;
    },
  });
}

export function useOverlayPlane(): OverlayPlane {
  return useContext(OverlayPlaneContext);
}

function planeRank(plane: OverlayPlane): number {
  return plane === "modal" ? 1 : 0;
}

function removeLayer(token: symbol): void {
  const index = openLayers.findIndex((layer) => layer.token === token);
  if (index >= 0) openLayers.splice(index, 1);
}

function pushLayer(token: symbol, plane: OverlayPlane): number {
  removeLayer(token);
  const order = nextOrder++;
  openLayers.push({ token, plane, order });
  return order;
}

function topmostLayer(): OpenLayer | undefined {
  return openLayers.reduce<OpenLayer | undefined>((topmost, candidate) => {
    if (!topmost) return candidate;
    const rank = planeRank(candidate.plane) - planeRank(topmost.plane);
    return rank > 0 || (rank === 0 && candidate.order > topmost.order)
      ? candidate
      : topmost;
  }, undefined);
}

export function createOverlayLayer(options: OverlayLayerOptions): OverlayLayer {
  const token = Symbol("wabou-overlay-layer");
  const plane = () => options.plane?.() ?? "floating";
  let wasOpen = options.open();
  let activePlane = plane();
  const [zIndex, setZIndex] = createSignal(
    wasOpen ? pushLayer(token, activePlane) : 0,
  );

  const restoreFocus = () => {
    if (options.restoreFocus?.() ?? true) options.returnFocus?.()?.focus();
  };

  createEffect(options.open, (open) => {
    const currentPlane = plane();
    if (open && !wasOpen) {
      setZIndex(pushLayer(token, currentPlane));
    } else if (!open && wasOpen) {
      removeLayer(token);
      restoreFocus();
    }
    wasOpen = open;
    activePlane = currentPlane;
  });
  createEffect(plane, (currentPlane) => {
    if (wasOpen && activePlane !== currentPlane) {
      setZIndex(pushLayer(token, currentPlane));
    }
    activePlane = currentPlane;
  });

  onCleanup(() => {
    removeLayer(token);
    if (wasOpen) restoreFocus();
    wasOpen = false;
  });

  const isTopmost = () => topmostLayer()?.token === token;

  return {
    plane,
    zIndex,
    isTopmost,
    onEscape(event) {
      if (
        !options.open() ||
        !isTopmost() ||
        event.key !== "Escape" ||
        options.closeOnEscape?.() === false
      )
        return;
      event.preventDefault();
      event.stopPropagation();
      options.onDismiss("escape");
    },
    onOutside(event) {
      if (
        !options.open() ||
        !isTopmost() ||
        options.closeOnOutside?.() === false
      )
        return;
      event.preventDefault();
      event.stopPropagation();
      options.onDismiss("outside");
    },
  };
}
