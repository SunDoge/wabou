import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ButtonProps } from "./button.mjs";
import { JSX } from "solid-js";
//#region src/components/carousel.d.ts
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
export declare function normalizeCarouselIndex(index: number, count: number, loop: boolean): number;
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
export declare function Carousel(props: CarouselProps): JSX.Element;
export interface CarouselContentProps extends Omit<ViewProps, "transform"> {
  trackClass?: string;
  dragThreshold?: number;
}
export declare function CarouselContent(props: CarouselContentProps): JSX.Element;
export interface CarouselItemProps extends ViewProps {
  "aria-label"?: string;
}
export declare function CarouselItem(props: CarouselItemProps): JSX.Element;
export declare function CarouselPrevious(props: Omit<ButtonProps, "children"> & {
  children?: JSX.Element;
}): JSX.Element;
export declare function CarouselNext(props: Omit<ButtonProps, "children"> & {
  children?: JSX.Element;
}): JSX.Element;
//#endregion
//# sourceMappingURL=carousel.d.mts.map