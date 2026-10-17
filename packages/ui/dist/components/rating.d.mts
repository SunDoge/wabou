import { clampRatingValue, normalizeRatingMax, ratingLabel } from "./rating-state.mjs";
import { JSX } from "solid-js";
//#region src/components/rating.d.ts
export interface RatingProps {
  value?: number;
  defaultValue?: number;
  max?: number;
  disabled?: boolean;
  readOnly?: boolean;
  allowClear?: boolean;
  label: string;
  class?: string;
  size?: number;
  onValueChange?: (value: number) => void;
}
export declare function Rating(props: RatingProps): JSX.Element;
//#endregion
export { clampRatingValue, normalizeRatingMax, ratingLabel };
//# sourceMappingURL=rating.d.mts.map