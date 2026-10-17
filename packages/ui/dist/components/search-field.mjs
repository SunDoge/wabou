import { Icon, View } from "../primitives/view.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { InputGroup, InputGroupButton, InputGroupInput } from "./forms.mjs";
import { Show, omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import x from "lucide-static/icons/x.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
//#region src/components/search-field.tsx
/** A native search input with consistent clear, Escape, and submit behavior. */
function SearchField(props) {
	const forwarded = omit(props, "value", "defaultValue", "onValueChange", "onSearch", "onClear", "clearLabel", "variant", "class", "surfaceClass", "inputClass", "inputRef");
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? "",
		onChange: props.onValueChange
	});
	let input;
	const clear = () => {
		if (!state.value() || props.disabled || props.readOnly) return false;
		if (!state.set("")) return false;
		props.onClear?.();
		input?.focus();
		return true;
	};
	return createComponent$1(InputGroup, {
		get "class"() {
			return props.class;
		},
		get variant() {
			return props.variant;
		},
		get surfaceClass() {
			return props.surfaceClass;
		},
		get children() {
			return [
				createComponent$1(View, {
					"aria-hidden": "true",
					class: "flex-none pl-2.5 flex items-center text-muted",
					get children() {
						return createComponent$1(Icon, {
							source: search,
							size: 15
						});
					}
				}),
				createComponent$1(InputGroupInput, mergeProps(forwarded, {
					ref: (node) => {
						input = node;
						props.inputRef?.(node);
					},
					get value() {
						return state.value();
					},
					get "class"() {
						return props.inputClass;
					},
					onInput: (event) => state.set(event.currentTarget.value),
					onKeyDown: (event) => {
						props.onKeyDown?.(event);
						if (event.key === "Escape" && clear()) event.preventDefault();
						if (event.key === "Enter") props.onSearch?.(state.value());
					}
				})),
				createComponent$1(Show, {
					get when() {
						return memo(() => {
							return !!(state.value() && !props.disabled);
						})() ? !props.readOnly : state.value() && !props.disabled;
					},
					get children() {
						return createComponent$1(InputGroupButton, {
							size: "icon",
							get "aria-label"() {
								return props.clearLabel ?? "Clear search";
							},
							onClick: clear,
							get children() {
								return createComponent$1(Icon, {
									source: x,
									"aria-hidden": "true",
									size: 15
								});
							}
						});
					}
				})
			];
		}
	});
}
//#endregion
export { SearchField };

//# sourceMappingURL=search-field.mjs.map