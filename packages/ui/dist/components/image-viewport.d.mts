import { ImageResourceErrorEvent, ImageResourceHandle, ImageResourceReadyEvent, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/image-viewport.d.ts
export interface ImageViewportSize {
  width: number;
  height: number;
}
export interface ImageViewportPoint {
  x: number;
  y: number;
}
export interface ImageViewportRect extends ImageViewportPoint {
  width: number;
  height: number;
}
export interface ImageViewportTransform {
  readonly viewport: ImageViewportSize;
  readonly image: ImageViewportSize;
  readonly frame: ImageViewportRect;
  readonly scale: number;
  imageToViewport(point: ImageViewportPoint): ImageViewportPoint;
  viewportToImage(point: ImageViewportPoint): ImageViewportPoint;
}
/** Deterministic contain + zoom + pan transform shared by paint and annotations. */
export declare function imageViewportTransform(options: {
  viewport: ImageViewportSize;
  image: ImageViewportSize;
  zoom?: number;
  pan?: ImageViewportPoint;
}): ImageViewportTransform;
export interface ImageViewportProps extends Omit<ViewProps, "children" | "ref" | "onWheel" | "onPointerDown" | "onPointerMove" | "onPointerUp" | "onPointerCancel"> {
  resource?: ImageResourceHandle;
  /** Intrinsic image size in image pixels. */
  imageSize?: ImageViewportSize;
  zoom?: number;
  pan?: ImageViewportPoint;
  pannable?: boolean;
  onPanChange?: (pan: ImageViewportPoint) => void;
  /** Optional replacement for the native Image, useful for generated media. */
  media?: JSX.Element;
  children?: JSX.Element;
  imageLabel?: string;
  onResourceReady?: (event: ImageResourceReadyEvent) => void;
  onResourceError?: (event: ImageResourceErrorEvent) => void;
}
/** A clipped image-space viewport with one explicit, reusable coordinate model. */
export declare function ImageViewport(props: ImageViewportProps): JSX.Element;
export interface AnnotationRegion extends ImageViewportRect {
  id: string;
  label?: string;
}
export declare function clampAnnotationRegion(region: AnnotationRegion, image: ImageViewportSize, minimumSize?: number): AnnotationRegion;
export interface AnnotationLayerProps extends Omit<ViewProps, "children" | "onPointerDown" | "onPointerMove" | "onPointerUp" | "onPointerCancel"> {
  regions: readonly AnnotationRegion[];
  selectedId?: string | null;
  minimumSize?: number;
  createRegionId?: () => string;
  interactionMode?: "edit" | "passthrough";
  onRegionsChange?: (regions: readonly AnnotationRegion[]) => void;
  onSelectedIdChange?: (id: string | null) => void;
}
/** Editable image-space regions composed above an ImageViewport. */
export declare function AnnotationLayer(props: AnnotationLayerProps): JSX.Element;
export interface ImageOverlayItem extends ImageViewportRect {
  id: string;
}
export interface ImageOverlayLayerProps<T extends ImageOverlayItem> extends Omit<ViewProps, "children"> {
  items: readonly T[];
  children: (item: T, index: () => number) => JSX.Element;
}
/** Read-only image-space overlays sharing the viewport's exact zoom and pan transform. */
export declare function ImageOverlayLayer<T extends ImageOverlayItem>(props: ImageOverlayLayerProps<T>): JSX.Element;
//#endregion
//# sourceMappingURL=image-viewport.d.mts.map