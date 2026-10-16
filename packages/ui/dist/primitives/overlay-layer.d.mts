import { JSX } from "solid-js";
//#region src/primitives/overlay-layer.d.ts
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
  returnFocus?: () => {
    focus(): void;
  } | undefined;
}
export interface OverlayLayer {
  plane(): OverlayPlane;
  /** Stable native sibling order for the current open lifetime. */
  zIndex(): number;
  isTopmost(): boolean;
  onEscape(event: DismissKeyEvent): void;
  onOutside(event: DismissEvent): void;
}
export interface OverlayPlaneProviderProps {
  plane: OverlayPlane;
  children?: JSX.Element;
}
/** Make nested portals inherit the current native stacking plane. */
export declare function OverlayPlaneProvider(props: OverlayPlaneProviderProps): JSX.Element;
export declare function useOverlayPlane(): OverlayPlane;
export declare function createOverlayLayer(options: OverlayLayerOptions): OverlayLayer;
//#endregion
//# sourceMappingURL=overlay-layer.d.mts.map