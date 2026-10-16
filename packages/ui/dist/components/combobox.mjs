import { Icon, Text } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { Popover } from "../primitives/popover.mjs";
import { componentsControlSize, componentsDisabledControlClass, componentsElevation, componentsSurfaceClass, useComponentsTheme } from "./theme.mjs";
import { pickerTriggerClass } from "./select-semantics.mjs";
import { Command } from "./command.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createSignal } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
//#region src/components/combobox.tsx
/** A searchable single-value picker built from Popover and Command. */
function Combobox(props) {
	const theme = useComponentsTheme();
	const [uncontrolledValue, setUncontrolledValue] = createSignal(props.defaultValue);
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const [query, setQuery] = createSignal("");
	let trigger;
	let search;
	const value = () => props.value ?? uncontrolledValue();
	const open = () => props.open ?? uncontrolledOpen();
	const selected = () => props.options.find((option) => option.value === value());
	const setOpen = (next) => {
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next);
		if (next) {
			setQuery("");
			requestAnimationFrame(() => search?.focus());
		} else requestAnimationFrame(() => trigger?.focus());
	};
	const select = (id) => {
		const option = props.options.find((candidate) => candidate.id === id);
		if (!option || option.disabled) return;
		if (props.value === void 0) setUncontrolledValue(option.value);
		option.onSelect?.();
		props.onValueChange?.(option.value);
		setOpen(false);
	};
	return createComponent$1(Popover, {
		contentRole: "presentation",
		popupRole: "listbox",
		get open() {
			return open();
		},
		onOpenChange: setOpen,
		placement: "bottom-start",
		get contentClass() {
			return mergeClasses("w-72 p-2", componentsSurfaceClass("floating"), props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		get motion() {
			return props.motion;
		},
		trigger: (popover) => createComponent$1(Button, {
			unstyled: true,
			role: "combobox",
			get disabled() {
				return props.disabled;
			},
			get "aria-label"() {
				return props["aria-label"];
			},
			"aria-haspopup": "listbox",
			get "aria-expanded"() {
				return open();
			},
			get "aria-valuetext"() {
				return selected()?.label;
			},
			ref: (node) => {
				trigger = node;
				popover.ref(node);
			},
			class: (state) => mergeClasses("w-72 overflow-hidden justify-between border", componentsControlSize("default"), pickerTriggerClass(props.triggerVariant ?? "default", state), props.class, componentsDisabledControlClass(state.disabled)),
			get onClick() {
				return popover.onClick;
			},
			get onKeyDown() {
				return popover.onKeyDown;
			},
			get children() {
				return [createComponent$1(Text, {
					get "class"() {
						return mergeClasses("min-w-0 flex-1 text-left truncate", selected() ? "text-primary" : "text-muted");
					},
					get children() {
						return selected()?.label ?? props.placeholder ?? "Select an option";
					}
				}), createComponent$1(Icon, {
					source: chevronDown,
					class: "flex-none text-muted",
					size: 16
				})];
			}
		}),
		get children() {
			return createComponent$1(Command, {
				get "aria-label"() {
					return `${props["aria-label"]} search`;
				},
				get query() {
					return query();
				},
				onQueryChange: setQuery,
				get placeholder() {
					return props.searchPlaceholder ?? "Search options";
				},
				get emptyText() {
					return props.emptyText;
				},
				get items() {
					return props.options;
				},
				onAction: select,
				onDismiss: () => setOpen(false),
				inputRef: (node) => search = node
			});
		}
	});
}
//#endregion
export { Combobox };

//# sourceMappingURL=combobox.mjs.map