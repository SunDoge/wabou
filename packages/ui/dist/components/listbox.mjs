import { Icon, Text, View } from "../primitives/view.mjs";
import { ScrollArea } from "../primitives/scroll-area.mjs";
import { componentsDisabledItemClass } from "./theme.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { moveMenuHighlight } from "./menu-state.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, createEffect, createSignal } from "solid-js";
import { applyRef, createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import { match } from "ts-pattern";
import check from "lucide-static/icons/check.svg?raw";
//#region src/components/listbox.tsx
const optionItems = (options) => options.map((option) => ({
	id: option.value,
	label: option.label,
	disabled: option.disabled
}));
/**
* Focusable inline single-selection list.
*
* Unlike Select, Listbox owns no popup or trigger. It is suitable for dialogs,
* inspectors and other surfaces where the choices are already visible.
*/
function Listbox(props) {
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue,
		onChange: (value) => value !== void 0 && props.onValueChange?.(value)
	});
	const [highlighted, setHighlighted] = createSignal();
	const items = () => optionItems(props.options);
	createEffect(() => ({
		options: props.options,
		selected: state.value(),
		highlighted: highlighted()
	}), ({ options, selected, highlighted: current }) => {
		const candidates = optionItems(options);
		if (candidates.some((item) => item.id === current && !item.disabled)) return;
		const selectedOption = candidates.find((item) => item.id === selected && !item.disabled);
		setHighlighted(selectedOption?.id ?? moveMenuHighlight(candidates, void 0, "first"));
	});
	const move = (direction) => {
		const next = moveMenuHighlight(items(), highlighted(), direction);
		if (next === void 0) return false;
		setHighlighted(next);
		return true;
	};
	const select = (value) => {
		const option = props.options.find((candidate) => candidate.value === value);
		if (!option || option.disabled) return false;
		state.set(option.value);
		props.onAction?.(option.value);
		return true;
	};
	const handleKeyDown = (event) => {
		if (match(event.key).with("ArrowDown", () => move("next")).with("ArrowUp", () => move("previous")).with("Home", () => move("first")).with("End", () => move("last")).with("Enter", () => select(highlighted())).with(" ", () => select(highlighted())).with("Escape", () => {
			props.onDismiss?.();
			return props.onDismiss !== void 0;
		}).otherwise(() => false)) event.preventDefault();
	};
	const visibleItems = () => Math.max(1, Math.min(props.options.length, props.maxVisibleItems ?? 8));
	const itemHeight = () => Math.max(24, props.itemHeight ?? 40);
	const viewportHeight = () => Math.max(1, props.viewportHeight ?? visibleItems() * itemHeight());
	return createComponent$1(ScrollArea, {
		get "class"() {
			return mergeClasses("w-full", props.fill ? "min-h-0 flex-1" : "flex-none", props.class);
		},
		get contentClass() {
			return mergeClasses("gap-1", props.listClass);
		},
		get style() {
			return memo(() => {
				return !!props.fill;
			})() ? void 0 : { height: `${viewportHeight()}px` };
		},
		get children() {
			return createComponent$1(View, {
				ref(r$) {
					var _ref$ = props.ref;
					typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : props.ref = r$;
				},
				role: "listbox",
				get "aria-label"() {
					return props["aria-label"];
				},
				get "aria-activedescendant"() {
					return highlighted();
				},
				focusOrder: 0,
				class: "select-none min-w-0 flex flex-col gap-1",
				onKeyDown: handleKeyDown,
				get children() {
					return memo(() => {
						return props.options.length === 0;
					})() ? createComponent$1(Text, {
						role: "status",
						class: "px-3 py-4 text-sm text-muted text-center",
						get children() {
							return props.emptyText ?? "No options available.";
						}
					}) : createComponent$1(For, {
						get each() {
							return props.options;
						},
						keyed: false,
						children: (option) => {
							const selected = () => state.value() === option().value;
							const active = () => highlighted() === option().value;
							return createComponent$1(View, {
								get id() {
									return option().value;
								},
								role: "option",
								get "aria-label"() {
									return option().accessibilityLabel ?? option().label;
								},
								get "aria-selected"() {
									return selected();
								},
								get "aria-disabled"() {
									return option().disabled;
								},
								get "class"() {
									return mergeClasses("w-full min-h-8 px-2 py-1 flex-none flex flex-row items-center gap-2 rounded-md", active() && !option().disabled ? "bg-control-hover text-primary" : "bg-transparent text-secondary", props.itemClass, componentsDisabledItemClass(option().disabled ?? false));
								},
								get style() {
									return { height: itemHeight() };
								},
								onPointerMove: () => !option().disabled && setHighlighted(option().value),
								onClick: () => select(option().value),
								get children() {
									return [
										memo(() => {
											return props.renderLeading?.(option());
										}),
										createComponent$1(View, {
											class: "min-w-0 flex-1 flex flex-col justify-center",
											get children() {
												return [createComponent$1(Text, {
													class: "min-w-0 truncate text-sm",
													get children() {
														return option().label;
													}
												}), memo(() => {
													return memo(() => {
														return !!option().description;
													})() ? createComponent$1(Text, {
														class: "min-w-0 truncate text-xs text-muted",
														get children() {
															return option().description;
														}
													}) : option().description;
												})];
											}
										}),
										memo(() => {
											return props.renderTrailing?.(option());
										}),
										createComponent$1(View, {
											"aria-hidden": "true",
											class: "w-4 h-4 flex-none",
											get children() {
												return memo(() => {
													return !!selected();
												})() ? createComponent$1(Icon, {
													source: check,
													class: "text-accent",
													size: 16
												}) : selected();
											}
										})
									];
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
export { Listbox };

//# sourceMappingURL=listbox.mjs.map