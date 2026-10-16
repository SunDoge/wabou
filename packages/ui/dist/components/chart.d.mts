import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/chart.d.ts
export interface ChartSeriesConfig {
  label: string;
  colorClass: string;
}
export type ChartConfig = Readonly<Record<string, ChartSeriesConfig>>;
export interface ChartContainerProps {
  config: ChartConfig;
  label: string;
  class?: string;
  style?: ViewProps["style"];
  children?: JSX.Element;
}
export declare function ChartContainer(props: ChartContainerProps): JSX.Element;
export declare function useChartConfig(): ChartConfig;
export declare function ChartLegend(props: {
  class?: string;
}): JSX.Element;
export declare function ChartEmpty(props: {
  message?: string;
  class?: string;
}): JSX.Element;
//#endregion
//# sourceMappingURL=chart.d.mts.map