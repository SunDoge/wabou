import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/switch.d.ts
export interface SwitchProps {
  checked?: boolean;
  defaultChecked?: boolean;
  disabled?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  label?: string;
  /** Place the label before or after the control in reading order. */
  labelPlacement?: "start" | "end";
  "aria-label"?: string;
  class?: string;
  size?: "sm" | "default";
}
declare function Switch$1(props: SwitchProps): JSX.Element;
//#endregion
export { Switch$1 as Switch };
//# sourceMappingURL=switch.d.mts.map