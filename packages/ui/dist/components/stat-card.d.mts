import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/stat-card.d.ts
export interface StatCardProps extends Omit<ViewProps, "children"> {
  label: string;
  value: string;
  description?: string;
  trend?: string;
  indicatorClass?: string;
}
export declare function StatCard(props: StatCardProps): JSX.Element;
//#endregion
//# sourceMappingURL=stat-card.d.mts.map