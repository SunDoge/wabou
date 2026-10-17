import { Icon } from "../primitives/view.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { InputGroup, InputGroupButton, InputGroupInput } from "./forms.mjs";
import { addNumberFieldStep, clampNumberFieldValue, normalizeNumberFieldRange, numberFieldValueFromEmpty } from "./number-field-state.mjs";
import { useHost } from "@wabou/core";
import { createEffect, createSignal, omit, untrack } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
import minus from "lucide-static/icons/minus.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
//#region src/components/number-field.tsx
/** Locale-aware numeric input with explicit native stepping semantics. */
function NumberField(props) {
	const host = useHost();
	const forwarded = omit(props, "value", "defaultValue", "min", "max", "step", "largeStep", "locale", "formatOptions", "placeholder", "changeOnWheel", "onValueChange", "class", "inputClass", "incrementLabel", "decrementLabel");
	const range = () => normalizeNumberFieldRange(props.min, props.max, props.step, props.largeStep);
	const locale = () => props.locale ?? host.intl.locale();
	const formatOptions = () => JSON.stringify(props.formatOptions ?? {});
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? null,
		disabled: () => Boolean(props.disabled || props.readOnly),
		onChange: props.onValueChange
	});
	const formattedValue = () => {
		const value = state.value();
		return value === null ? "" : host.intl.formatNumber(locale(), value, formatOptions());
	};
	const [focused, setFocused] = createSignal(false);
	const [draft, setDraft] = createSignal(untrack(formattedValue));
	createEffect(() => ({
		focused: focused(),
		value: formattedValue()
	}), ({ focused: isFocused, value }) => {
		if (!isFocused) setDraft(value);
	});
	const update = (next) => {
		if (props.disabled || props.readOnly) return false;
		const normalized = next === null ? null : clampNumberFieldValue(next, range());
		const changed = state.set(normalized);
		setDraft(formattedValue());
		return changed;
	};
	const commitDraft = () => {
		const value = draft().trim();
		if (!value) {
			update(null);
			return;
		}
		let parsed = NaN;
		try {
			parsed = host.intl.parseNumber(locale(), value, formatOptions());
		} catch {}
		if (Number.isFinite(parsed)) update(parsed);
		else setDraft(formattedValue());
	};
	const changeBy = (direction, amount) => {
		const current = state.value();
		const next = current === null ? numberFieldValueFromEmpty(direction, range()) : addNumberFieldStep(current, direction * amount, range());
		update(next);
	};
	const canDecrement = () => {
		const value = state.value();
		return !props.disabled && !props.readOnly && (value === null || value > range().min);
	};
	const canIncrement = () => {
		const value = state.value();
		return !props.disabled && !props.readOnly && (value === null || value < range().max);
	};
	return createComponent$1(InputGroup, {
		get "class"() {
			return props.class;
		},
		get children() {
			return [
				createComponent$1(InputGroupButton, {
					size: "icon",
					class: "w-7 h-7 mx-0.5",
					get disabled() {
						return !canDecrement();
					},
					get "aria-label"() {
						return props.decrementLabel ?? `Decrease ${props["aria-label"]}`;
					},
					onClick: () => changeBy(-1, range().step),
					get children() {
						return createComponent$1(Icon, {
							source: minus,
							"aria-hidden": "true",
							size: 14
						});
					}
				}),
				createComponent$1(InputGroupInput, mergeProps(forwarded, {
					role: "spinbutton",
					get "aria-label"() {
						return props["aria-label"];
					},
					get "aria-valuemin"() {
						return memo(() => {
							return !!Number.isFinite(range().min);
						})() ? range().min : void 0;
					},
					get "aria-valuemax"() {
						return memo(() => {
							return !!Number.isFinite(range().max);
						})() ? range().max : void 0;
					},
					get "aria-valuenow"() {
						return state.value() ?? void 0;
					},
					get "aria-valuetext"() {
						return memo(() => {
							return state.value() === null;
						})() ? void 0 : formattedValue();
					},
					get value() {
						return draft();
					},
					get placeholder() {
						return props.placeholder;
					},
					get "class"() {
						return props.inputClass;
					},
					onFocus: (event) => {
						setFocused(true);
						props.onFocus?.(event);
					},
					onBlur: (event) => {
						commitDraft();
						setFocused(false);
						props.onBlur?.(event);
					},
					onInput: (event) => {
						const next = event.currentTarget.value;
						setDraft(next);
						let parsed = NaN;
						try {
							parsed = host.intl.parseNumber(locale(), next, formatOptions());
						} catch {
							return;
						}
						if (Number.isFinite(parsed) && parsed >= range().min && parsed <= range().max) state.set(parsed);
					},
					onKeyDown: (event) => {
						props.onKeyDown?.(event);
						if (event.defaultPrevented || props.disabled || props.readOnly) return;
						if (match(event.key).with("ArrowUp", () => changeBy(1, range().step)).with("ArrowDown", () => changeBy(-1, range().step)).with("PageUp", () => changeBy(1, range().largeStep)).with("PageDown", () => changeBy(-1, range().largeStep)).with("Home", () => Number.isFinite(range().min) ? update(range().min) : false).with("End", () => Number.isFinite(range().max) ? update(range().max) : false).otherwise(() => false)) event.preventDefault();
					},
					onWheel: (event) => {
						props.onWheel?.(event);
						if (event.defaultPrevented || !props.changeOnWheel || !focused() || props.disabled || props.readOnly || event.deltaY === 0) return;
						changeBy(event.deltaY < 0 ? 1 : -1, range().step);
						event.preventDefault();
					}
				})),
				createComponent$1(InputGroupButton, {
					size: "icon",
					class: "w-7 h-7 mx-0.5",
					get disabled() {
						return !canIncrement();
					},
					get "aria-label"() {
						return props.incrementLabel ?? `Increase ${props["aria-label"]}`;
					},
					onClick: () => changeBy(1, range().step),
					get children() {
						return createComponent$1(Icon, {
							source: plus,
							"aria-hidden": "true",
							size: 14
						});
					}
				})
			];
		}
	});
}
//#endregion
export { NumberField };

//# sourceMappingURL=number-field.mjs.map