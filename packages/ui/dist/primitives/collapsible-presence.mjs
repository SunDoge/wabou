import { useReducedMotion } from "../animation/config.mjs";
import { createTransition } from "../animation.mjs";
import { View } from "./view.mjs";
import { createMeasuredSize } from "./measure.mjs";
import { createPresence } from "./presence.mjs";
import { number, px } from "@wabou/core/style";
import { Show, createEffect, untrack } from "solid-js";
import { applyRef, createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/primitives/collapsible-presence.tsx
/**
* Measured disclosure content with explicit presence and subtree isolation.
* Height participates in layout while a subtree opacity layer masks glyphs
* crossing the moving clip edge.
*/
function CollapsiblePresence(props) {
	const inheritedReducedMotion = useReducedMotion();
	const reducedMotion = () => props.reducedMotion ?? inheritedReducedMotion();
	const open = () => props.open;
	const initiallyOpen = untrack(open);
	const presence = createPresence(open);
	let initialMeasurement = true;
	let heightTransition;
	const opacityTransition = createTransition(() => open() ? 1 : 0, {
		duration: props.duration ?? .2,
		ease: props.ease ?? "easeOut",
		reducedMotion
	});
	const measured = createMeasuredSize({ onChange(size) {
		if (initialMeasurement && initiallyOpen && !props.animateInitial) heightTransition?.jump(size.height);
		initialMeasurement = false;
	} });
	const transitionOptions = () => ({
		duration: props.duration ?? .2,
		ease: props.ease ?? "easeOut",
		reducedMotion
	});
	heightTransition = createTransition(() => open() && measured.measured() ? measured.height() : 0, {
		...transitionOptions(),
		onComplete(value) {
			const isOpen = untrack(open);
			if (value === 0 && !isOpen) presence.finishExit();
			else if (isOpen) presence.finishEnter();
		}
	});
	createEffect(() => [
		open(),
		measured.measured(),
		measured.height()
	], ([isOpen, isMeasured, height]) => {
		if (isOpen && isMeasured && height === 0) presence.finishEnter();
	});
	const style = () => ({
		...props.style,
		height: px(heightTransition?.value() ?? 0),
		opacity: number(opacityTransition.value())
	});
	return createComponent$1(View, {
		get "class"() {
			return props.class;
		},
		classList: { "overflow-hidden": true },
		get style() {
			return style();
		},
		get interactionBlocked() {
			return !open();
		},
		get "aria-hidden"() {
			return open() ? void 0 : "true";
		},
		get children() {
			return createComponent$1(Show, {
				get when() {
					return presence.mounted();
				},
				get children() {
					return createComponent$1(View, {
						ref(r$) {
							var _ref$ = measured.ref;
							typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
						},
						get children() {
							return createComponent$1(View, mergeProps(() => {
								return props.contentProps;
							}, {
								get "class"() {
									return props.contentClass;
								},
								get style() {
									return props.contentStyle;
								},
								get children() {
									return props.children;
								}
							}));
						}
					});
				}
			});
		}
	});
}
//#endregion
export { CollapsiblePresence };

//# sourceMappingURL=collapsible-presence.mjs.map