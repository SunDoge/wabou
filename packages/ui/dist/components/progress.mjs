import { useReducedMotion } from "../animation/config.mjs";
import { createNativeLoopAnimation } from "../animation.mjs";
import { NativeWidget, Svg, Text, View } from "../primitives/view.mjs";
import { Spinner } from "./display.mjs";
import { finiteOr } from "./range.mjs";
import { progressCircleSource } from "./progress-circle-source.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, createContext, createMemo, omit, useContext } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/progress.tsx
const ProgressContext = createContext();
function useProgressContext() {
	const context = useContext(ProgressContext);
	if (!context) throw new Error("Progress parts must be used inside ProgressRoot");
	return context;
}
function normalizeProgressValue(value, minValue, maxValue) {
	const min = finiteOr(minValue, 0);
	const requestedMax = finiteOr(maxValue, 100);
	const max = requestedMax > min ? requestedMax : min + 1;
	const normalizedValue = Math.max(min, Math.min(max, finiteOr(value, min)));
	return {
		value: normalizedValue,
		min,
		max,
		percent: (normalizedValue - min) / (max - min) * 100
	};
}
/** Semantic progress state with explicit, composable visual parts. */
function ProgressRoot(props) {
	const forwarded = omit(props, "value", "minValue", "maxValue", "indeterminate", "label", "getValueLabel", "children", "class");
	const details = () => normalizeProgressValue(props.value, props.minValue, props.maxValue);
	const indeterminate = () => props.indeterminate ?? false;
	const defaultValueLabel = () => `${Math.round(details().percent)} percent`;
	const context = {
		value: () => details().value,
		min: () => details().min,
		max: () => details().max,
		percent: () => details().percent,
		indeterminate,
		label: () => props.label ?? "Progress",
		valueLabel: () => indeterminate() ? void 0 : props.getValueLabel?.(details()) ?? defaultValueLabel()
	};
	return createComponent$1(ProgressContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				role: "progressbar",
				get "aria-label"() {
					return context.label();
				},
				get "aria-valuemin"() {
					return context.min();
				},
				get "aria-valuemax"() {
					return context.max();
				},
				get "aria-valuenow"() {
					return memo(() => {
						return !!indeterminate();
					})() ? void 0 : context.value();
				},
				get "aria-valuetext"() {
					return context.valueLabel();
				},
				get "class"() {
					return mergeClasses("w-full min-w-0 flex flex-col gap-2", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
const progressTrackSize = (size) => size === "xs" ? "h-1" : size === "sm" ? "h-1.5" : size === "lg" ? "h-2.5" : "h-2";
function ProgressTrack(props) {
	const forwarded = omit(props, "size");
	return createComponent$1(View, mergeProps(forwarded, {
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("w-full flex-none overflow-hidden rounded-full bg-control", progressTrackSize(props.size), props.class);
		}
	}));
}
function IndeterminateProgressFill(props) {
	const reducedMotion = useReducedMotion();
	const animation = createNativeLoopAnimation({
		duration: 1.35,
		reducedMotion
	});
	return createComponent$1(NativeWidget, mergeProps(props, {
		tag: "progress-indeterminate",
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("w-full h-full flex-none text-accent", props.class);
		},
		get config() {
			return { animation: animation() };
		}
	}));
}
function ProgressFill(props) {
	const context = useProgressContext();
	return createComponent$1(Show, {
		get when() {
			return !context.indeterminate();
		},
		get fallback() {
			return createComponent$1(IndeterminateProgressFill, props);
		},
		get children() {
			return createComponent$1(View, mergeProps(props, {
				"aria-hidden": "true",
				get "class"() {
					return mergeClasses("h-full rounded-full bg-accent", context.percent() < 100 && "rounded-r-none", props.class);
				},
				get style() {
					return {
						width: `${context.percent()}%`,
						...props.style
					};
				}
			}));
		}
	});
}
function ProgressLabel(props) {
	const context = useProgressContext();
	return createComponent$1(Text, mergeProps(props, {
		get "class"() {
			return mergeClasses("min-w-0 text-sm text-secondary", props.class);
		},
		get children() {
			return props.children ?? context.label();
		}
	}));
}
function ProgressValueLabel(props) {
	const context = useProgressContext();
	return createComponent$1(Text, mergeProps(props, {
		get "class"() {
			return mergeClasses("flex-none text-sm font-mono text-muted", props.class);
		},
		get children() {
			return props.children ?? context.valueLabel() ?? "In progress";
		}
	}));
}
/** Compact progress bar; use ProgressRoot and parts for custom composition. */
function Progress(props) {
	const forwarded = omit(props, "class", "size");
	return createComponent$1(ProgressRoot, mergeProps(forwarded, { get children() {
		return createComponent$1(ProgressTrack, {
			get "class"() {
				return props.class;
			},
			get size() {
				return props.size;
			},
			get children() {
				return createComponent$1(ProgressFill, {});
			}
		});
	} }));
}
const progressCircleSize = (size) => size === "xs" ? "w-3 h-3" : size === "sm" ? "w-4 h-4" : size === "lg" ? "w-6 h-6" : "w-5 h-5";
/** Compact circular progress indicator using the same semantic range contract. */
function ProgressCircle(props) {
	const forwarded = omit(props, "class", "size");
	const details = createMemo(() => normalizeProgressValue(props.value, props.minValue, props.maxValue));
	const source = createMemo(() => progressCircleSource(props.indeterminate ? 30 : details().percent));
	const graphic = () => createComponent$1(Svg, {
		get source() {
			return source();
		},
		"aria-hidden": "true",
		class: "absolute inset-0 w-full h-full flex-none"
	});
	return createComponent$1(ProgressRoot, mergeProps(forwarded, {
		get "class"() {
			return mergeClasses("relative flex-none items-center justify-center gap-0 text-accent", progressCircleSize(props.size), props.class);
		},
		get children() {
			return createComponent$1(Show, {
				get when() {
					return props.indeterminate;
				},
				get fallback() {
					return graphic();
				},
				get children() {
					return createComponent$1(Spinner, {
						decorative: true,
						class: "absolute inset-0 w-full h-full"
					});
				}
			});
		}
	}));
}
//#endregion
export { Progress, ProgressCircle, ProgressFill, ProgressLabel, ProgressRoot, ProgressTrack, ProgressValueLabel, normalizeProgressValue };

//# sourceMappingURL=progress.mjs.map