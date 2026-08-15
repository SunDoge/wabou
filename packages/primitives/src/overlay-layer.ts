import { createEffect, onCleanup } from "solid-js";

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
  onDismiss: (reason: OverlayDismissReason) => void;
  closeOnEscape?: () => boolean;
  closeOnOutside?: () => boolean;
  restoreFocus?: () => boolean;
  returnFocus?: () => { focus(): void } | undefined;
}

export interface OverlayLayer {
  isTopmost(): boolean;
  onEscape(event: DismissKeyEvent): void;
  onOutside(event: DismissEvent): void;
}

// A QuickJS window owns one module graph, so this stack is naturally scoped to
// one native window. Tokens follow open order rather than component mount order:
// reopening an older overlay correctly promotes it above its siblings.
const openLayers: symbol[] = [];

function removeLayer(token: symbol): void {
  const index = openLayers.lastIndexOf(token);
  if (index >= 0) openLayers.splice(index, 1);
}

export function createOverlayLayer(options: OverlayLayerOptions): OverlayLayer {
  const token = Symbol("wabou-overlay-layer");
  let wasOpen = options.open();
  if (wasOpen) openLayers.push(token);

  const restoreFocus = () => {
    if (options.restoreFocus?.() ?? true) options.returnFocus?.()?.focus();
  };

  createEffect(options.open, (open) => {
    if (open && !wasOpen) {
      removeLayer(token);
      openLayers.push(token);
    } else if (!open && wasOpen) {
      removeLayer(token);
      restoreFocus();
    }
    wasOpen = open;
  });

  onCleanup(() => {
    removeLayer(token);
    if (wasOpen) restoreFocus();
    wasOpen = false;
  });

  const isTopmost = () => openLayers.at(-1) === token;

  return {
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
