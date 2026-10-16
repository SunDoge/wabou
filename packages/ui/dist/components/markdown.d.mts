import { JSX } from "solid-js";
//#region src/components/markdown.d.ts
export type MarkdownVariant = "document" | "conversation" | "prompt";
export interface MarkdownProps {
  source: string;
  /** Repair an incomplete Markdown tail while text is still arriving. */
  streaming?: boolean;
  /** Document typography by default; conversation and prompt stay message-sized. */
  variant?: MarkdownVariant;
  class?: string;
  "aria-label"?: string;
}
/** Parses GFM in JavaScript and renders native Wabou components, without HTML or a DOM. */
export declare function Markdown(props: MarkdownProps): JSX.Element;
//#endregion
//# sourceMappingURL=markdown.d.mts.map