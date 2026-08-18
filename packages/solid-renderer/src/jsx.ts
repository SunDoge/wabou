import type { WabouStyle } from "../../style/src/index";
import type { Element as SolidElement } from "solid-js";
import type {
  Handle,
  WabouBuiltinIntrinsicElements,
  WabouElementProps,
  WabouIntrinsicElements,
} from "./index";

/** Renderer-owned JSX namespace for Solid 2's automatic JSX type lookup. */
export namespace JSX {
  export type Element = SolidElement | Handle | readonly Element[];
  export type CSSProperties = WabouStyle;

  export type ElementClass = {};
  export type ElementAttributesProperty = {};
  export interface ElementChildrenAttribute {
    children: {};
  }

  export interface IntrinsicElements
    extends WabouBuiltinIntrinsicElements,
      WabouIntrinsicElements {}
  export type IntrinsicAttributes = {};
}

// Keep source compatibility for packages that import `JSX` from `solid-js`.
declare module "solid-js" {
  namespace JSX {
    type Element = import("./jsx").JSX.Element;
    type CSSProperties = import("./jsx").JSX.CSSProperties;
    interface ElementClass {}
    interface ElementAttributesProperty {}
    interface ElementChildrenAttribute {
      children: {};
    }
    interface IntrinsicElements
      extends WabouBuiltinIntrinsicElements,
        WabouIntrinsicElements {}
    interface IntrinsicAttributes {}
  }
}

// The Solid compiler erases JSX rather than calling these functions. They are
// exported only so TypeScript can resolve the standard jsx-runtime entry.
export function jsx(): never {
  throw new Error("Wabou JSX must be compiled by the Solid transform");
}
export const jsxs = jsx;
export const jsxDEV = jsx;
export const Fragment = (props: WabouElementProps) => props.children;
