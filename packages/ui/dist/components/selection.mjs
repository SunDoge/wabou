import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { Center } from "../primitives/layout.mjs";
import { componentsControlSize, componentsDisabledInteractiveClass } from "./theme.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { createRovingFocus } from "../primitives/interactions/roving-focus.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, createContext, onCleanup, useContext } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import { match } from "ts-pattern";
import check from "lucide-static/icons/check.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
//#region src/components/selection.tsx
function selectionGeometry(size) {
	return match(size).with("sm", () => ({
		controlWithLabel: "min-h-7 px-1 items-start gap-2 rounded-md border border-transparent",
		controlWithoutLabel: "w-9 h-7 p-0 items-center justify-center gap-0 rounded-md border border-transparent",
		indicator: "w-3.5 h-3.5",
		label: "text-xs",
		iconSize: 10,
		radioDot: "w-1.5 h-1.5"
	})).with("default", () => ({
		controlWithLabel: "min-h-7 px-1 items-start gap-2 rounded-md border border-transparent",
		controlWithoutLabel: "w-10 h-7 p-0 items-center justify-center gap-0 rounded-md border border-transparent",
		indicator: "w-4 h-4",
		label: "text-sm",
		iconSize: 12,
		radioDot: "w-2 h-2"
	})).with("lg", () => ({
		controlWithLabel: "min-h-10 px-1 items-start gap-2 rounded-md border border-transparent",
		controlWithoutLabel: "w-10 h-10 p-0 items-center justify-center gap-0 rounded-md border border-transparent",
		indicator: "w-[18px] h-[18px]",
		label: "text-base",
		iconSize: 14,
		radioDot: "w-2.5 h-2.5"
	})).exhaustive();
}
function selectionControlClass(hasLabel, size) {
	const geometry = selectionGeometry(size);
	return hasLabel ? geometry.controlWithLabel : geometry.controlWithoutLabel;
}
function selectionLabelClass(size) {
	return mergeClasses("min-w-0 flex-1 whitespace-normal text-left", selectionGeometry(size).label);
}
function Checkbox(props) {
	const size = () => props.size ?? "default";
	const state = createControllableState({
		value: () => props.checked,
		defaultValue: props.defaultChecked ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onCheckedChange
	});
	const checked = state.value;
	const toggle = () => {
		state.set(!checked());
	};
	const boxColors = () => match({
		checked: checked(),
		indeterminate: !!props.indeterminate
	}).with({ checked: true }, () => "bg-accent border-accent text-on-accent").with({ indeterminate: true }, () => "bg-accent border-accent text-on-accent").otherwise(() => "bg-input border-strong text-primary");
	const ariaChecked = () => match({
		checked: checked(),
		indeterminate: !!props.indeterminate
	}).with({ indeterminate: true }, () => "mixed").otherwise(({ checked }) => checked);
	const indicator = () => match({
		checked: checked(),
		indeterminate: !!props.indeterminate
	}).with({ indeterminate: true }, () => minus).with({ checked: true }, () => check).otherwise(() => void 0);
	return createComponent$1(Button, {
		unstyled: true,
		role: "checkbox",
		get disabled() {
			return props.disabled;
		},
		get "aria-label"() {
			return props["aria-label"] ?? props.label;
		},
		get "aria-checked"() {
			return ariaChecked();
		},
		get selected() {
			return checked();
		},
		class: (buttonState) => mergeClasses(selectionControlClass(!!props.label, size()), !buttonState.disabled && buttonState.hovered && "bg-control-hover", buttonState.focusVisible && "border-focus", props.class, componentsDisabledInteractiveClass(buttonState.disabled)),
		onClick: toggle,
		get children() {
			return [createComponent$1(Center, {
				"aria-hidden": "true",
				get "class"() {
					return mergeClasses(selectionGeometry(size()).indicator, "flex-none border", "rounded text-xs font-bold", boxColors());
				},
				get children() {
					return memo(() => {
						return !!indicator();
					})() ? createComponent$1(Icon, {
						get source() {
							return indicator();
						},
						get size() {
							return selectionGeometry(size()).iconSize;
						},
						class: "text-on-accent"
					}) : indicator();
				}
			}), memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent$1(Text, {
					get "class"() {
						return mergeClasses(selectionLabelClass(size()), "text-primary");
					},
					get children() {
						return props.label;
					}
				}) : props.label;
			})];
		}
	});
}
const RadioContext = createContext();
function RadioGroup(props) {
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue,
		disabled: () => props.disabled ?? false,
		onChange: (value) => value !== void 0 && props.onValueChange?.(value)
	});
	const value = state.value;
	const select = (next) => {
		state.set(next);
	};
	const roving = createRovingFocus({
		orientation: () => props.orientation ?? "vertical",
		loop: props.loop,
		preferred: (id) => value() === id,
		onMove: select
	});
	return createComponent(RadioContext, {
		value: {
			value,
			select,
			disabled: () => props.disabled ?? false,
			appearance: () => props.appearance ?? "radio",
			size: () => props.size ?? "default",
			register: (id, target, disabled) => roving.register({
				id,
				target,
				disabled
			}),
			activate: (id) => {
				roving.activate(id);
			},
			isTabStop: roving.isTabStop,
			move: roving.move
		},
		get children() {
			return createComponent$1(View, {
				role: "radiogroup",
				get "aria-label"() {
					return props["aria-label"];
				},
				get "class"() {
					return mergeClasses("flex", props.orientation === "horizontal" ? "flex-row" : "flex-col", props.appearance === "segment" ? "items-center gap-0.5 rounded-md border border-subtle bg-control p-0.5" : "gap-3", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function RadioGroupItem(props) {
	const group = useContext(RadioContext);
	if (!group) throw new Error("RadioGroupItem must be used inside RadioGroup");
	const checked = () => group.value() === props.value;
	const disabled = () => group.disabled() || (props.disabled ?? false);
	const size = () => props.size ?? group.size();
	const segmentSize = () => match(size()).with("sm", () => "h-7 min-w-0 flex-1 px-2 items-center justify-center rounded-sm border border-transparent text-xs font-medium").with("default", () => "h-8 min-w-0 flex-1 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium").with("lg", () => "h-10 min-w-0 flex-1 px-4 items-center justify-center rounded-sm border border-transparent text-base font-medium").exhaustive();
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent$1(Button, {
		unstyled: true,
		role: "radio",
		get disabled() {
			return disabled();
		},
		get selected() {
			return checked();
		},
		get "aria-label"() {
			return props["aria-label"] ?? props.label;
		},
		get "aria-checked"() {
			return checked();
		},
		get focusOrder() {
			return group.isTabStop(props.value) ? 0 : -1;
		},
		ref: (node) => {
			unregister?.();
			unregister = group.register(props.value, node, disabled);
		},
		class: (buttonState) => mergeClasses(group.appearance() === "segment" ? segmentSize() : selectionControlClass(!!props.label, size()), group.appearance() === "segment" && checked() ? "bg-selected text-primary shadow-xs" : !buttonState.disabled && buttonState.hovered && "bg-control-hover", buttonState.focusVisible && "border-focus", props.class, componentsDisabledInteractiveClass(buttonState.disabled)),
		onClick: () => group.select(props.value),
		onFocus: () => group.activate(props.value),
		onKeyDown: (event) => {
			if (group.move(props.value, event.key)) event.preventDefault();
		},
		get children() {
			return [memo(() => {
				return memo(() => {
					return group.appearance() === "radio";
				})() && createComponent$1(Center, {
					"aria-hidden": "true",
					get "class"() {
						return mergeClasses(selectionGeometry(size()).indicator, "flex-none border", "rounded-full bg-input", match(checked()).with(true, () => "border-accent").with(false, () => "border-strong").exhaustive());
					},
					get children() {
						return memo(() => {
							return !!checked();
						})() ? createComponent$1(View, { get "class"() {
							return mergeClasses(selectionGeometry(size()).radioDot, "rounded-full bg-accent");
						} }) : checked();
					}
				});
			}), memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent$1(Text, {
					get "class"() {
						return mergeClasses(selectionLabelClass(size()), checked() ? "text-primary" : "text-secondary");
					},
					get children() {
						return props.label;
					}
				}) : props.label;
			})];
		}
	});
}
function Toggle(props) {
	const state = createControllableState({
		value: () => props.pressed,
		defaultValue: props.defaultPressed ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onPressedChange
	});
	const pressed = state.value;
	const toggle = () => {
		state.set(!pressed());
	};
	const size = () => match(props.size ?? "default").with("sm", () => `${componentsControlSize("sm")} min-w-7`).with("default", () => `${componentsControlSize("default")} min-w-8`).with("lg", () => `${componentsControlSize("lg")} min-w-10`).exhaustive();
	const colors = (state) => match({
		selected: pressed(),
		hovered: !state.disabled && state.hovered
	}).with({ selected: true }, () => "bg-selected border-accent text-primary").with({ hovered: true }, () => "bg-control-hover text-primary").otherwise(() => "bg-transparent text-secondary");
	return createComponent$1(Button, {
		unstyled: true,
		get disabled() {
			return props.disabled;
		},
		get selected() {
			return pressed();
		},
		get "aria-label"() {
			return props["aria-label"];
		},
		get "aria-pressed"() {
			return pressed();
		},
		class: (state) => mergeClasses("items-center justify-center border font-medium", size(), colors(state), match(props.variant ?? "default").with("outline", () => "border-strong").with("default", () => "border-transparent").exhaustive(), state.focusVisible && "border-focus", props.class, componentsDisabledInteractiveClass(state.disabled)),
		onClick: toggle,
		get children() {
			return props.children;
		}
	});
}
const ToggleGroupContext = createContext();
function nextToggleGroupValue(current, value, type) {
	if (type === "single") return current === value ? "" : value;
	const values = Array.isArray(current) ? current : [];
	return values.includes(value) ? values.filter((candidate) => candidate !== value) : [...values, value];
}
/** Shadcn-style single-value toggle group with native roving focus. */
function ToggleGroup(props) {
	const type = () => props.type ?? "single";
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? (props.type === "multiple" ? [] : ""),
		disabled: () => props.disabled ?? false,
		onChange: (value) => {
			if (props.type === "multiple") props.onValueChange?.(Array.isArray(value) ? value : []);
			else props.onValueChange?.(typeof value === "string" ? value : "");
		}
	});
	const activateFromKeyboard = (value) => {
		if (type() === "single" && state.value() !== value) state.set(value);
	};
	const roving = createRovingFocus({
		orientation: () => "horizontal",
		loop: props.loop,
		preferred: (id) => {
			const current = state.value();
			return Array.isArray(current) ? current.includes(id) : current === id;
		},
		onMove: activateFromKeyboard
	});
	const context = {
		selected(value) {
			const current = state.value();
			return Array.isArray(current) ? current.includes(value) : current === value;
		},
		disabled: () => props.disabled ?? false,
		toggle: (value) => state.set(nextToggleGroupValue(state.value(), value, type())),
		register(value, node, disabled) {
			return roving.register({
				id: value,
				target: node,
				disabled
			});
		},
		activate(value) {
			roving.activate(value);
		},
		isTabStop: roving.isTabStop,
		move: roving.move,
		variant: () => props.variant ?? "default",
		size: () => props.size ?? "default",
		segmented: () => props.segmented ?? false
	};
	return createComponent(ToggleGroupContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				role: "group",
				get "aria-label"() {
					return props["aria-label"];
				},
				"aria-orientation": "horizontal",
				get "aria-disabled"() {
					return props.disabled;
				},
				get "class"() {
					return mergeClasses("flex flex-row items-stretch rounded-md", props.segmented ? "gap-0 overflow-hidden border border-strong bg-surface" : mergeClasses("bg-transparent", match(props.spacing ?? 0).with(0, () => "gap-0").with(1, () => "gap-1").with(2, () => "gap-2").exhaustive()), props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function ToggleGroupItem(props) {
	const group = useContext(ToggleGroupContext);
	if (!group) throw new Error("ToggleGroupItem must be used inside ToggleGroup");
	const selected = () => group.selected(props.value);
	const disabled = () => group.disabled() || (props.disabled ?? false);
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent$1(Button, {
		unstyled: true,
		get disabled() {
			return disabled();
		},
		get selected() {
			return selected();
		},
		get "aria-label"() {
			return props["aria-label"];
		},
		get "aria-pressed"() {
			return selected();
		},
		get focusOrder() {
			return group.isTabStop(props.value) ? 0 : -1;
		},
		ref: (node) => {
			unregister?.();
			unregister = group.register(props.value, node, disabled);
		},
		class: (state) => mergeClasses("h-7 flex-1 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium", match(props.size ?? group.size()).with("sm", () => componentsControlSize("sm")).with("default", () => componentsControlSize("default")).with("lg", () => componentsControlSize("lg")).exhaustive(), match({
			selected: selected(),
			accent: props.variant === "accent",
			hovered: !state.disabled && state.hovered
		}).with({
			selected: true,
			accent: true
		}, () => "bg-accent text-on-accent").with({ selected: true }, () => "bg-selected text-primary").with({ hovered: true }, () => "bg-control-hover text-primary").otherwise(() => "bg-transparent text-muted"), group.segmented() && "rounded-none border-transparent", !group.segmented() && (props.variant ?? group.variant()) === "outline" && "border-strong", state.focusVisible && "border-focus", props.class, componentsDisabledInteractiveClass(state.disabled)),
		onFocus: () => group.activate(props.value),
		onClick: () => group.toggle(props.value),
		onKeyDown: (event) => {
			if (group.move(props.value, event.key)) event.preventDefault();
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
export { Checkbox, RadioGroup, RadioGroupItem, Toggle, ToggleGroup, ToggleGroupItem, nextToggleGroupValue, selectionGeometry };

//# sourceMappingURL=selection.mjs.map