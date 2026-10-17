import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/alert.d.ts
export type AlertVariant = "default" | "info" | "success" | "warning" | "error" | "destructive";
export type AlertSize = "sm" | "default" | "lg";
export declare function alertColors(variant: AlertVariant): {
  container: string;
  title: string;
  description: string;
};
export interface AlertProps extends Omit<ViewProps, "role"> {
  variant?: AlertVariant;
  size?: AlertSize;
  /** Optional leading graphic. Pass false to suppress the semantic default. */
  icon?: JSX.Element | false;
  /** Edge-to-edge status strip without container chrome. */
  banner?: boolean;
  /** Show a close control and delegate visibility to the owner. */
  onClose?: () => void;
  /** Convenience form; compound usage can render AlertTitle directly. */
  title?: string;
}
/** A native status callout with shadcn-compatible compound composition. */
export declare function Alert(props: AlertProps): JSX.Element;
export declare function AlertTitle(props: TextProps): JSX.Element;
export declare function AlertDescription(props: TextProps): JSX.Element;
/** Recovery and acknowledgement controls belonging to an alert. */
export declare function AlertActions(props: ViewProps): JSX.Element;
//#endregion
//# sourceMappingURL=alert.d.mts.map