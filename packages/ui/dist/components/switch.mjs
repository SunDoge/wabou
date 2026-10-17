import { useReducedMotion } from "../animation/config.mjs";
import "../animation.mjs";
import { View, translate2d as translate2d$1 } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { componentsDisabledInteractiveClass } from "./theme.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { Label } from "./label.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, memo } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/switch.tsx
function switchColors(checked, state) {
	if (state.disabled) return checked ? "bg-accent" : "bg-control";
	return match({
		checked,
		pressed: state.pressed,
		hovered: state.hovered
	}).with({
		checked: true,
		pressed: true
	}, () => "bg-accent-pressed").with({
		checked: true,
		hovered: true
	}, () => "bg-accent-hover").with({ checked: true }, () => "bg-accent").with({
		checked: false,
		pressed: true
	}, () => "bg-control-pressed").with({
		checked: false,
		hovered: true
	}, () => "bg-control-hover").with({ checked: false }, () => "bg-control").exhaustive();
}
function switchGeometry(size) {
	return size === "sm" ? {
		trackClass: "w-7 h-4 p-0.5",
		thumbClass: "w-3 h-3",
		travel: 12
	} : {
		trackClass: "w-9 h-5 p-0.5",
		thumbClass: "w-4 h-4",
		travel: 16
	};
}
function switchTrackClass(checked, state, size = "default") {
	return mergeClasses("flex-none overflow-hidden rounded-full", switchGeometry(size).trackClass, switchColors(checked, state));
}
function switchControlClass(state) {
	return mergeClasses("w-10 h-6 p-0 flex-none items-center justify-center rounded-full border border-transparent bg-transparent", state.focusVisible && "border-focus");
}
function Switch(props) {
	const state = createControllableState({
		value: () => props.checked,
		defaultValue: props.defaultChecked ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onCheckedChange
	});
	const checked = state.value;
	const reducedMotion = useReducedMotion();
	const size = () => props.size ?? "default";
	const geometry = () => switchGeometry(size());
	let control;
	const toggle = () => {
		state.set(!checked());
	};
	return createComponent(View, {
		get "class"() {
			return mergeClasses("w-full min-w-0 flex items-start gap-2", props.labelPlacement === "start" && "flex-row-reverse", props.class);
		},
		get children() {
			return [createComponent(Button, {
				ref: (node) => {
					control = node;
				},
				unstyled: true,
				role: "switch",
				get disabled() {
					return props.disabled;
				},
				get "aria-label"() {
					return props["aria-label"] ?? props.label;
				},
				get "aria-checked"() {
					return checked();
				},
				class: (state) => mergeClasses(switchControlClass(state), componentsDisabledInteractiveClass(state.disabled)),
				onClick: toggle,
				renderContent: (buttonState) => createComponent(View, {
					"aria-hidden": "true",
					get "class"() {
						return switchTrackClass(checked(), buttonState(), size());
					},
					get children() {
						return createComponent(View, {
							get "class"() {
								return mergeClasses(geometry().thumbClass, "rounded-full bg-surface");
							},
							get transform() {
								return translate2d$1(checked() ? geometry().travel : 0, 0);
							},
							get nativeMotion() {
								return {
									kind: "spring",
									response: reducedMotion() ? 0 : .16,
									damping: 1,
									epsilon: .02,
									targetTransform: translate2d$1(checked() ? geometry().travel : 0, 0)
								};
							}
						});
					}
				})
			}), memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent(Label, {
					control: () => control,
					get disabled() {
						return props.disabled;
					},
					class: "min-w-0 flex-1 select-none whitespace-normal text-sm font-normal text-primary",
					onClick: (event) => {
						if (props.disabled || event.defaultPrevented) return;
						toggle();
					},
					get children() {
						return props.label;
					}
				}) : props.label;
			})];
		}
	});
}
//#endregion
export { Switch, switchControlClass, switchGeometry, switchTrackClass };

//# sourceMappingURL=switch.mjs.map