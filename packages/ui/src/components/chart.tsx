import {
  createComponent,
  createContext,
  For as ForValue,
  type JSX,
  useContext,
} from "solid-js";
import { Text, View, type ViewProps } from "../primitives";
import { mergeClasses } from "@wabou/core/style";

export interface ChartSeriesConfig {
  label: string;
  colorClass: string;
}

export type ChartConfig = Readonly<Record<string, ChartSeriesConfig>>;

const ChartContext = createContext<ChartConfig>({});

export interface ChartContainerProps {
  config: ChartConfig;
  label: string;
  class?: string;
  style?: ViewProps["style"];
  children?: JSX.Element;
}

export function ChartContainer(props: ChartContainerProps): JSX.Element {
  return createComponent(ChartContext, {
    get value() {
      return props.config;
    },
    get children() {
      return (
        <View
          role="img"
          aria-label={props.label}
          class={mergeClasses("relative min-w-0", props.class)}
          style={props.style}
        >
          {props.children}
        </View>
      );
    },
  });
}

export function useChartConfig(): ChartConfig {
  return useContext(ChartContext);
}

export function ChartLegend(props: { class?: string }): JSX.Element {
  const config = useChartConfig();
  return (
    <View
      class={mergeClasses(
        "flex flex-row flex-wrap items-center gap-4",
        props.class,
      )}
    >
      <ForValue each={Object.values(config)}>
        {(series) => (
          <View class="flex flex-row items-center gap-2">
            <View
              aria-hidden="true"
              class={mergeClasses(
                "w-2.5 h-2.5 flex-none rounded-full",
                series.colorClass,
              )}
            />
            <Text class="text-sm text-secondary">{series.label}</Text>
          </View>
        )}
      </ForValue>
    </View>
  );
}

export function ChartEmpty(props: {
  message?: string;
  class?: string;
}): JSX.Element {
  return (
    <View class={mergeClasses("h-48 items-center justify-center", props.class)}>
      <Text class="text-sm text-muted">{props.message ?? "No chart data"}</Text>
    </View>
  );
}
