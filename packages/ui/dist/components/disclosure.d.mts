import { Easing } from "../animation.mjs";
import { ViewProps } from "../primitives/view.mjs";
import { ButtonProps } from "../primitives/button.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/disclosure.d.ts
export interface CollapsibleProps extends Omit<ViewProps, "children" | "class"> {
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  reducedMotion?: boolean;
  onOpenChange?: (open: boolean) => void;
  class?: string;
}
export declare function Collapsible(props: CollapsibleProps): import("@wabou/core/jsx-runtime").JSX.Element;
export interface CollapsibleTriggerProps extends Omit<ButtonProps, "children" | "class" | "unstyled"> {
  children?: JSX.Element;
  /** Render the built-in trailing chevron. Disable when children provide one. */
  indicator?: boolean;
  class?: string;
}
export declare function CollapsibleTrigger(props: CollapsibleTriggerProps): import("@wabou/core/jsx-runtime").JSX.Element;
export interface CollapsibleContentProps extends ViewProps {
  duration?: number;
  ease?: Easing;
  animateInitial?: boolean;
}
export declare function CollapsibleContent(props: CollapsibleContentProps): import("@wabou/core/jsx-runtime").JSX.Element;
export type AccordionType = "single" | "multiple";
type AccordionValue = string | readonly string[];
export declare function nextAccordionValue(current: AccordionValue, type: AccordionType, item: string, collapsible?: boolean): AccordionValue;
export interface AccordionProps extends Omit<ViewProps, "children" | "class"> {
  children?: JSX.Element;
  type?: AccordionType;
  value?: AccordionValue;
  defaultValue?: AccordionValue;
  collapsible?: boolean;
  disabled?: boolean;
  reducedMotion?: boolean;
  onValueChange?: (value: AccordionValue) => void;
  class?: string;
}
export declare function Accordion(props: AccordionProps): import("@wabou/core/jsx-runtime").JSX.Element;
export interface AccordionItemProps extends Omit<ViewProps, "children" | "class"> {
  value: string;
  disabled?: boolean;
  children?: JSX.Element;
  class?: string;
}
export declare function AccordionItem(props: AccordionItemProps): import("@wabou/core/jsx-runtime").JSX.Element;
export interface AccordionTriggerProps extends Omit<ButtonProps, "children" | "class" | "unstyled"> {
  children?: JSX.Element;
  class?: string;
}
export declare function AccordionTrigger(props: AccordionTriggerProps): import("@wabou/core/jsx-runtime").JSX.Element;
export type AccordionContentProps = ViewProps;
export declare function AccordionContent(props: AccordionContentProps): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
export {};
//# sourceMappingURL=disclosure.d.mts.map