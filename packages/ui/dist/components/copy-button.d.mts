import { ButtonProps } from "./button.mjs";
import { JSX } from "solid-js";
//#region src/components/copy-button.d.ts
export interface CopyButtonProps extends Omit<ButtonProps, "onClick"> {
  value: string;
  idleLabel?: string;
  copiedLabel?: string;
  copiedChildren?: JSX.Element;
  onCopied?: () => void;
  onCopyError?: (error: unknown) => void;
}
export declare function CopyButton(props: CopyButtonProps): JSX.Element;
//#endregion
//# sourceMappingURL=copy-button.d.mts.map