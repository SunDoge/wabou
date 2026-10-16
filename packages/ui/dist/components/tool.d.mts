import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { CodeBlockProps } from "./code-block.mjs";
import { CollapsibleContentProps, CollapsibleProps, CollapsibleTriggerProps } from "./disclosure.mjs";
import { JSX } from "solid-js";
//#region src/components/tool.d.ts
export type ToolStatus = "pending" | "running" | "success" | "failed";
export interface ToolProps extends CollapsibleProps {}
/** Composable disclosure root for an AI or automation tool invocation. */
export declare function Tool(props: ToolProps): JSX.Element;
export interface ToolHeaderProps extends Omit<CollapsibleTriggerProps, "children" | "class"> {
  title: string;
  summary?: string;
  status?: ToolStatus;
  icon?: string;
  class?: string;
}
export declare function toolHeaderLabel(title: string, summary?: string, status?: ToolStatus): string;
/** Stable title, summary, status and disclosure geometry for one tool call. */
export declare function ToolHeader(props: ToolHeaderProps): JSX.Element;
export declare function ToolContent(props: CollapsibleContentProps): JSX.Element;
export interface ToolCodeSectionProps extends Omit<ViewProps, "children" | "class"> {
  code: string;
  label: string;
  language?: string;
  copyable?: boolean;
  class?: string;
  codeClass?: string;
  codeProps?: Omit<CodeBlockProps, "code" | "language" | "copyable">;
}
/** Labelled code payload used for tool parameters, results and errors. */
export declare function ToolCodeSection(props: ToolCodeSectionProps): JSX.Element;
export interface ToolInputProps extends Omit<ToolCodeSectionProps, "label"> {
  label?: string;
}
export declare function ToolInput(props: ToolInputProps): JSX.Element;
export interface ToolOutputProps extends Omit<ToolCodeSectionProps, "label"> {
  label?: string;
  error?: boolean;
}
export declare function ToolOutput(props: ToolOutputProps): JSX.Element;
//#endregion
//# sourceMappingURL=tool.d.mts.map