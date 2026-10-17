import { useReducedMotion } from "../animation/config.mjs";
import { createNativeLoopAnimation, createSweep } from "../animation.mjs";
import { NativeWidget, Text, View } from "../primitives/view.mjs";
import { createMeasuredSize } from "../primitives/measure.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createSignal, omit } from "solid-js";
import { applyRef, createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/display.tsx
function Skeleton(props) {
	const reducedMotion = useReducedMotion();
	const motionDisabled = () => props.animated === false || reducedMotion();
	const [width, setWidth] = createSignal(0, { ownedWrite: true });
	const measured = createMeasuredSize({ onChange: (size) => setWidth(size.width) });
	const sweep = createSweep({
		extent: width,
		itemRatio: .4,
		duration: 1.6,
		ease: "easeInOut",
		reducedMotion: motionDisabled,
		reducedValue: .5
	});
	return createComponent$1(View, {
		ref(r$) {
			var _ref$ = measured.ref;
			typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
		},
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("overflow-hidden rounded-md bg-control", props.class);
		},
		get children() {
			return createComponent$1(View, {
				class: "w-2/5 h-full flex-none bg-control-hover",
				get transform() {
					return sweep.transform();
				},
				get style() {
					return { opacity: motionDisabled() ? 0 : 1 };
				}
			});
		}
	});
}
function Spinner(props) {
	const reducedMotion = useReducedMotion();
	const animation = createNativeLoopAnimation({
		duration: () => props.duration ?? .9,
		speed: () => props.speed ?? 1,
		paused: () => props.paused ?? false,
		reducedMotion
	});
	return createComponent$1(NativeWidget, {
		tag: "spinner",
		get role() {
			return props.decorative ? void 0 : "status";
		},
		get "aria-hidden"() {
			return props.decorative ? true : void 0;
		},
		get "aria-label"() {
			return memo(() => {
				return !!props.decorative;
			})() ? void 0 : props.label ?? "Loading";
		},
		get "class"() {
			return mergeClasses("w-4 h-4 flex-none text-accent", props.class);
		},
		get config() {
			return { animation: animation() };
		}
	});
}
function Kbd(props) {
	const forwarded = omit(props, "class", "style", "children", "maxLines");
	return createComponent$1(View, mergeProps(forwarded, {
		get role() {
			return props.role ?? "label";
		},
		get "class"() {
			return mergeClasses("h-5 min-w-5 px-1 inline-flex items-center justify-center flex-none rounded bg-control text-xs font-medium text-muted", props.class);
		},
		get style() {
			return {
				"line-height": 1,
				...props.style
			};
		},
		get children() {
			return createComponent$1(Text, {
				"aria-hidden": "true",
				maxLines: 1,
				class: "flex-none whitespace-nowrap",
				get children() {
					return props.children;
				}
			});
		}
	}));
}
function KbdGroup(props) {
	const forwarded = omit(props, "class", "children");
	return createComponent$1(View, mergeProps(forwarded, {
		get "class"() {
			return mergeClasses("inline-flex items-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { Kbd, KbdGroup, Skeleton, Spinner };

//# sourceMappingURL=display.mjs.map