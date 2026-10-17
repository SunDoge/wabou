import { JSX } from "solid-js";
//#region src/components/slider.d.ts
export interface SliderProps {
  value?: number;
  defaultValue?: number;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  /** Direction of the track and pointer interaction. */
  orientation?: "horizontal" | "vertical";
  /** Fill from the thumb toward the maximum end without changing values. */
  reversed?: boolean;
  label: string;
  valueText?: (value: number) => string;
  onValueChange?: (value: number) => void;
  class?: string;
}
export declare function Slider(props: SliderProps): JSX.Element;
//#endregion
//# sourceMappingURL=slider.d.mts.map