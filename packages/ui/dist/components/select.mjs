import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { Popover } from "../primitives/popover.mjs";
import { ScrollArea } from "../primitives/scroll-area.mjs";
import { componentsControlSize, componentsDisabledControlClass, componentsElevation, useComponentsTheme } from "./theme.mjs";
import { createSelectInteraction } from "../primitives/interactions/select.mjs";
import { pickerOptionClass, pickerTriggerClass, selectControlsId } from "./select-semantics.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, createUniqueId } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import { match } from "ts-pattern";
import check from "lucide-static/icons/check.svg?raw";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
//#region src/components/select.tsx
const ITEM_HEIGHT = 36;
const VISIBLE_ITEMS = 6;
/** Shadcn-inspired single Select backed by Wabou-native interaction state. */
function Select(props) {
	const theme = useComponentsTheme();
	const id = createUniqueId();
	let trigger;
	let content;
	let viewport;
	let scrollTop = 0;
	const items = () => props.options.map((option) => ({
		id: option.value,
		textValue: option.label,
		disabled: option.disabled
	}));
	const execute = (command) => {
		match(command).with({ type: "FOCUS_TRIGGER" }, () => requestAnimationFrame(() => trigger?.focus())).with({ type: "FOCUS_CONTENT" }, () => requestAnimationFrame(() => content?.focus())).with({ type: "SCROLL_TO_ITEM" }, ({ id }) => {
			const index = props.options.findIndex((option) => option.value === id);
			if (index < 0) return;
			const firstVisible = Math.floor(scrollTop / ITEM_HEIGHT);
			const lastVisible = firstVisible + VISIBLE_ITEMS - 1;
			const nextTop = index < firstVisible ? index * ITEM_HEIGHT : index > lastVisible ? (index - VISIBLE_ITEMS + 1) * ITEM_HEIGHT : scrollTop;
			if (nextTop !== scrollTop) {
				scrollTop = nextTop;
				requestAnimationFrame(() => viewport?.scrollTo({ top: nextTop }));
			}
		}).exhaustive();
	};
	const interaction = createSelectInteraction({
		items,
		value: () => props.value,
		defaultValue: props.defaultValue,
		open: () => props.open,
		defaultOpen: props.defaultOpen,
		disabled: () => props.disabled ?? false,
		onValueChange: props.onValueChange,
		onOpenChange: props.onOpenChange,
		execute
	});
	const selected = () => props.options.find((option) => option.value === interaction.value());
	const handleKeyDown = (event) => {
		if (match(event.key).with("ArrowDown", () => interaction.send({ type: "ARROW_DOWN" })).with("ArrowUp", () => interaction.send({ type: "ARROW_UP" })).with("Home", () => interaction.send({ type: "HOME" })).with("End", () => interaction.send({ type: "END" })).with("Enter", () => interaction.send({ type: interaction.open() ? "SELECT" : "OPEN" })).with(" ", () => interaction.send({ type: interaction.open() ? "SELECT" : "OPEN" })).with("Escape", () => interaction.send({ type: "CLOSE" })).otherwise((key) => interaction.typeahead(key))) {
			event.preventDefault();
			if (event.key === "Escape") event.stopPropagation?.();
		}
	};
	return createComponent$1(Popover, {
		contentRole: "presentation",
		popupRole: "listbox",
		get open() {
			return interaction.open();
		},
		onOpenChange: (open) => {
			interaction.send({ type: open ? "OPEN" : "CLOSE" });
		},
		placement: "bottom-start",
		openOnPointerDown: true,
		get contentClass() {
			return mergeClasses("w-72 p-1 rounded-lg border border-subtle bg-surface", props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		get motion() {
			return props.motion ?? false;
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
				return interaction.open();
			},
			get "aria-controls"() {
				return selectControlsId(`${id}-listbox`, interaction.open());
			},
			get "aria-valuetext"() {
				return selected()?.label;
			},
			ref: (node) => {
				trigger = node;
				props.ref?.(node);
				popover.ref(node);
			},
			class: (state) => mergeClasses("w-72 overflow-hidden justify-between border", componentsControlSize("default"), pickerTriggerClass(props.triggerVariant ?? "default", state), props.class, componentsDisabledControlClass(state.disabled)),
			get onClick() {
				return popover.onClick;
			},
			get onPointerDown() {
				return popover.onPointerDown;
			},
			get onPointerCancel() {
				return popover.onPointerCancel;
			},
			onKeyDown: (event) => {
				popover.onKeyDown(event);
				handleKeyDown(event);
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
			return createComponent$1(ScrollArea, {
				ref: (node) => {
					viewport = node;
					scrollTop = 0;
				},
				class: "w-full flex-none",
				contentClass: "gap-1",
				get style() {
					return { height: `${Math.max(1, Math.min(props.options.length, VISIBLE_ITEMS)) * ITEM_HEIGHT - 4}px` };
				},
				onScroll: (event) => {
					scrollTop = event.scrollY ?? scrollTop;
				},
				get children() {
					return createComponent$1(View, {
						id: `${id}-listbox`,
						ref: (node) => content = node,
						role: "listbox",
						get "aria-label"() {
							return props["aria-label"];
						},
						get "aria-activedescendant"() {
							return memo(() => {
								return !!interaction.highlighted();
							})() ? `${id}-option-${interaction.highlighted()}` : void 0;
						},
						focusOrder: 0,
						class: "select-none min-w-0 flex flex-col gap-1",
						onKeyDown: handleKeyDown,
						get children() {
							return createComponent$1(For, {
								get each() {
									return props.options;
								},
								keyed: false,
								children: (option) => {
									const selected = () => interaction.value() === option().value;
									const highlighted = () => interaction.highlighted() === option().value;
									return createComponent$1(View, {
										get id() {
											return `${id}-option-${option().value}`;
										},
										role: "option",
										get "aria-selected"() {
											return selected();
										},
										get "aria-disabled"() {
											return option().disabled;
										},
										get "class"() {
											return mergeClasses("w-full h-8 flex-none px-2.5 flex items-center justify-between gap-2 rounded-md text-sm leading-normal", pickerOptionClass(option().disabled ?? false, highlighted()));
										},
										onPointerMove: () => {
											if (option().disabled) return;
											interaction.send({
												type: "HIGHLIGHT",
												id: option().value
											});
										},
										onClick: () => interaction.send({
											type: "SELECT",
											id: option().value
										}),
										get children() {
											return [createComponent$1(Text, {
												class: "min-w-0 flex-1 text-sm truncate",
												get children() {
													return option().label;
												}
											}), createComponent$1(View, {
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
											})];
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
export { Select };

//# sourceMappingURL=select.mjs.map