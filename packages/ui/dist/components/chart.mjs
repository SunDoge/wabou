import { Text, View } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, createComponent, createContext, useContext } from "solid-js";
import { createComponent as createComponent$1 } from "@wabou/core/renderer";
//#region src/components/chart.tsx
const ChartContext = createContext({});
function ChartContainer(props) {
	return createComponent(ChartContext, {
		get value() {
			return props.config;
		},
		get children() {
			return createComponent$1(View, {
				role: "img",
				get "aria-label"() {
					return props.label;
				},
				get "class"() {
					return mergeClasses("relative min-w-0", props.class);
				},
				get style() {
					return props.style;
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function useChartConfig() {
	return useContext(ChartContext);
}
function ChartLegend(props) {
	const config = useChartConfig();
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("flex flex-row flex-wrap items-center gap-4", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return Object.values(config);
				},
				children: (series) => createComponent$1(View, {
					class: "flex flex-row items-center gap-2",
					get children() {
						return [createComponent$1(View, {
							"aria-hidden": "true",
							get "class"() {
								return mergeClasses("w-2.5 h-2.5 flex-none rounded-full", series.colorClass);
							}
						}), createComponent$1(Text, {
							class: "text-sm text-secondary",
							get children() {
								return series.label;
							}
						})];
					}
				})
			});
		}
	});
}
function ChartEmpty(props) {
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("h-48 items-center justify-center", props.class);
		},
		get children() {
			return createComponent$1(Text, {
				class: "text-sm text-muted",
				get children() {
					return props.message ?? "No chart data";
				}
			});
		}
	});
}
//#endregion
export { ChartContainer, ChartEmpty, ChartLegend, useChartConfig };

//# sourceMappingURL=chart.mjs.map