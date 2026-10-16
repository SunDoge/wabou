import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/property-list.d.ts
export interface PropertyListProps extends Omit<ViewProps, "class"> {
  class?: string;
}
export declare function PropertyList(props: PropertyListProps): JSX.Element;
export interface PropertyRowProps {
  name: string;
  value: string;
  class?: string;
  nameClass?: string;
  valueClass?: string;
}
/** Compact two-column data intended for inspectors and settings summaries. */
export declare function PropertyRow(props: PropertyRowProps): JSX.Element;
//#endregion
//# sourceMappingURL=property-list.d.mts.map