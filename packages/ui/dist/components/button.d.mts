import { ButtonProps as ButtonProps$1 } from "../primitives/button.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/button.d.ts
export type ButtonVariant = "default" | "secondary" | "outline" | "ghost" | "destructive";
export type ButtonSize = "sm" | "default" | "lg" | "icon";
export interface ButtonProps extends Omit<ButtonProps$1, "variant" | "tone" | "unstyled"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  class?: string;
  style?: ButtonProps$1["style"];
  /** Disable activation and replace the leading content with a native spinner. */
  loading?: boolean;
  /** Visible label used while loading. Defaults to the ordinary children. */
  loadingLabel?: string;
}
export declare function Button(props: ButtonProps): JSX.Element;
//#endregion
//# sourceMappingURL=button.d.mts.map