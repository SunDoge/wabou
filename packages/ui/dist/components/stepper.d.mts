import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/stepper.d.ts
export interface StepperStep {
  id: string;
  label: string;
  description?: string;
}
export interface StepperProps extends Omit<ViewProps, "children"> {
  steps: readonly StepperStep[];
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
}
export declare function Stepper(props: StepperProps): JSX.Element;
//#endregion
//# sourceMappingURL=stepper.d.mts.map