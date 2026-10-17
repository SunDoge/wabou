import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/code-block.d.ts
export interface CodeBlockProps extends Omit<ViewProps, "children"> {
  code: string;
  language?: string;
  copyable?: boolean;
  copyLabel?: string;
  copiedLabel?: string;
}
export declare function CodeBlock(props: CodeBlockProps): JSX.Element;
//#endregion
//# sourceMappingURL=code-block.d.mts.map