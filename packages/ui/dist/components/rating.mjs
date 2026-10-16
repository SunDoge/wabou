import { useReducedMotion } from "../animation/config.mjs";
import { createTransition } from "../animation.mjs";
import { Icon, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { componentsDisabledInteractiveClass } from "./theme.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { createRovingFocus } from "../primitives/interactions/roving-focus.mjs";
import { clampRatingValue, normalizeRatingMax, ratingLabel } from "./rating-state.mjs";
import { mergeClasses, scale2d } from "@wabou/core/style";
import { For, createSignal, onCleanup } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import { P, match } from "ts-pattern";
import star from "lucide-static/icons/star.svg?raw";
//#region src/components/rating.tsx
function RatingIcon(props) {
	const reducedMotion = useReducedMotion();
	const emphasis = createTransition(() => props.highlighted ? 1 : 0, {
		duration: .12,
		ease: "easeOut",
		reducedMotion
	});
	return createComponent$1(Icon, {
		"aria-hidden": "true",
		source: star,
		get size() {
			return props.size;
		},
		get fill() {
			return props.highlighted ? "currentColor" : "none";
		},
		get "class"() {
			return props.highlighted ? "text-accent" : "text-muted";
		},
		get transform() {
			return scale2d(.9 + emphasis.value() * .1);
		}
	});
}
function Rating(props) {
	const max = () => normalizeRatingMax(props.max);
	const normalize = (value) => clampRatingValue(value, max());
	const state = createControllableState({
		value: () => props.value === void 0 ? void 0 : normalize(props.value),
		defaultValue: normalize(props.defaultValue),
		disabled: () => (props.disabled ?? false) || (props.readOnly ?? false),
		onChange: props.onValueChange
	});
	const value = () => normalize(state.value());
	const [preview, setPreview] = createSignal();
	let previewGeneration = 0;
	const previewItem = (item) => {
		previewGeneration++;
		setPreview(item);
	};
	const clearPreviewAfterPointerDispatch = () => {
		const generation = ++previewGeneration;
		queueMicrotask(() => {
			if (generation === previewGeneration) setPreview(void 0);
		});
	};
	const shownValue = () => preview() ?? value();
	const disabled = () => props.disabled ?? false;
	const inert = () => disabled() || (props.readOnly ?? false);
	const items = () => Array.from({ length: max() }, (_, index) => index + 1);
	const select = (next) => {
		if (inert()) return;
		const normalized = normalize(next);
		state.set(props.allowClear && normalized === value() ? 0 : normalized);
	};
	const roving = createRovingFocus({
		orientation: () => "horizontal",
		onMove: (id) => select(Number(id))
	});
	return createComponent$1(View, {
		role: "radiogroup",
		get "aria-label"() {
			return props.label;
		},
		get "aria-disabled"() {
			return disabled() || void 0;
		},
		"aria-orientation": "horizontal",
		get "class"() {
			return mergeClasses("flex flex-col items-start gap-1.5", props.class, componentsDisabledInteractiveClass(disabled()));
		},
		get children() {
			return createComponent$1(View, {
				class: "flex flex-row items-center gap-0.5",
				get children() {
					return createComponent$1(For, {
						get each() {
							return items();
						},
						children: (item) => {
							const checked = () => value() === item;
							const highlighted = () => item <= shownValue();
							let unregister;
							onCleanup(() => unregister?.());
							return createComponent$1(Button, {
								unstyled: true,
								role: "radio",
								get "aria-label"() {
									return ratingLabel(item);
								},
								get "aria-checked"() {
									return checked();
								},
								get selected() {
									return checked();
								},
								get disabled() {
									return disabled();
								},
								get focusOrder() {
									return memo(() => {
										return !!disabled();
									})() ? -1 : checked() || value() === 0 && item === 1 ? 0 : -1;
								},
								ref: (node) => {
									unregister?.();
									unregister = roving.register({
										id: String(item),
										target: node,
										disabled: inert
									});
								},
								class: (buttonState) => mergeClasses("w-8 h-8 items-center justify-center rounded-md border border-transparent", match({
									focused: buttonState.focusVisible,
									hovered: buttonState.hovered
								}).with({ focused: true }, () => "border-focus bg-control").with({ hovered: true }, () => "bg-control-hover").with({
									focused: false,
									hovered: false
								}, () => "bg-transparent").exhaustive()),
								onPointerEnter: () => !inert() && previewItem(item),
								onPointerLeave: clearPreviewAfterPointerDispatch,
								onClick: () => select(item),
								onKeyDown: (event) => {
									if (match(event.key).with(P.union("Home", "End", "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"), () => roving.move(String(item), event.key)).otherwise(() => false)) event.preventDefault();
								},
								get children() {
									return createComponent$1(RatingIcon, {
										get highlighted() {
											return highlighted();
										},
										get size() {
											return props.size ?? 20;
										}
									});
								}
							});
						}
					});
				}
			});
		}
	});
}
//#endregion
export { Rating, clampRatingValue, normalizeRatingMax, ratingLabel };

//# sourceMappingURL=rating.mjs.map