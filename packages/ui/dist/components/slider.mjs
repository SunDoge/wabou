import { NativeWidget } from "../primitives/view.mjs";
import { componentsDisabledInteractiveClass } from "./theme.mjs";
import { decimalPlaces, finiteOr, normalizeRange } from "./range.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createSignal } from "solid-js";
import { createComponent as createComponent$1 } from "@wabou/core/renderer";
//#region src/components/slider.tsx
function Slider(props) {
	const orientation = () => props.orientation ?? "horizontal";
	const range = () => normalizeRange(props.min, props.max, props.step);
	const min = () => range().min;
	const max = () => range().max;
	const step = () => range().step;
	const clamp = (value) => Math.max(min(), Math.min(max(), finiteOr(value, min())));
	const snap = (value) => {
		const stepped = min() + Math.round((clamp(value) - min()) / step()) * step();
		const precision = decimalPlaces(step());
		return clamp(Number(stepped.toFixed(precision)));
	};
	const [local, setLocal] = createSignal(snap(props.defaultValue ?? min()));
	const value = () => snap(props.value ?? local());
	const update = (next) => {
		if (props.disabled) return;
		const normalized = snap(next);
		const changed = normalized !== value();
		if (props.value === void 0) setLocal(normalized);
		if (changed) props.onValueChange?.(normalized);
	};
	const changeBy = (amount) => update(value() + amount);
	const onKeyDown = (event) => {
		if (props.disabled) return;
		if (event.key === "ArrowLeft" || event.key === "ArrowDown") changeBy(-step());
		else if (event.key === "ArrowRight" || event.key === "ArrowUp") changeBy(step());
		else if (event.key === "PageDown") changeBy(-step() * 10);
		else if (event.key === "PageUp") changeBy(step() * 10);
		else if (event.key === "Home") update(min());
		else if (event.key === "End") update(max());
		else return;
		event.preventDefault();
	};
	return createComponent$1(NativeWidget, {
		tag: "slider",
		role: "slider",
		get "aria-label"() {
			return props.label;
		},
		get "aria-valuemin"() {
			return min();
		},
		get "aria-valuemax"() {
			return max();
		},
		get "aria-valuenow"() {
			return value();
		},
		get "aria-valuetext"() {
			return props.valueText?.(value()) ?? String(value());
		},
		get "aria-disabled"() {
			return props.disabled ?? false;
		},
		get "aria-orientation"() {
			return orientation();
		},
		get focusOrder() {
			return props.disabled ? -1 : 0;
		},
		get "class"() {
			return mergeClasses(orientation() === "vertical" ? "w-7 h-[120px] select-none" : "w-full h-7 select-none", "text-accent", props.disabled ? "" : "cursor-pointer", props.class, componentsDisabledInteractiveClass(props.disabled ?? false));
		},
		get config() {
			return {
				min: min(),
				max: max(),
				step: step(),
				value: value(),
				disabled: props.disabled ?? false,
				orientation: orientation(),
				reversed: props.reversed ?? false
			};
		},
		onChange: (event) => update(event.value),
		onKeyDown
	});
}
//#endregion
export { Slider };

//# sourceMappingURL=slider.mjs.map