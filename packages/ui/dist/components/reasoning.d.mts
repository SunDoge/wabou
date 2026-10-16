import { CollapsibleContentProps, CollapsibleProps, CollapsibleTriggerProps } from "./disclosure.mjs";
import { JSX } from "solid-js";
//#region src/components/reasoning.d.ts
export interface ReasoningProps extends CollapsibleProps {}
/** Quiet disclosure surface for model reasoning or intermediate thought. */
export declare function Reasoning(props: ReasoningProps): JSX.Element;
export interface ReasoningTriggerProps extends Omit<CollapsibleTriggerProps, "children" | "class"> {
  label?: string;
  streaming?: boolean;
  class?: string;
}
export declare function ReasoningTrigger(props: ReasoningTriggerProps): JSX.Element;
export declare function ReasoningContent(props: CollapsibleContentProps): JSX.Element;
//#endregion
//# sourceMappingURL=reasoning.d.mts.map