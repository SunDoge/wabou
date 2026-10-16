import { useReducedMotion } from "../animation/config.mjs";
import "../animation.mjs";
import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { CollapsiblePresence } from "../primitives/collapsible-presence.mjs";
import { isSelected, toggleSelection } from "../primitives/interactions/selection.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { createDisclosure } from "../primitives/interactions/disclosure.mjs";
import { createRovingFocus } from "../primitives/interactions/roving-focus.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createContext, omit, onCleanup, useContext } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
//#region src/components/disclosure.tsx
function DisclosureIndicator(props) {
	return createComponent$1(View, {
		class: "w-4 h-4 flex-none",
		"aria-hidden": "true",
		get children() {
			return createComponent$1(Icon, {
				get source() {
					return props.open() ? chevronDown : chevronRight;
				},
				class: "text-muted",
				size: 16
			});
		}
	});
}
const CollapsibleContext = createContext();
const useCollapsible = () => {
	const value = useContext(CollapsibleContext);
	if (!value) throw new Error("Collapsible parts must be inside Collapsible");
	return value;
};
function Collapsible(props) {
	const inheritedReducedMotion = useReducedMotion();
	const state = createDisclosure({
		open: () => props.open,
		defaultOpen: props.defaultOpen,
		disabled: () => props.disabled ?? false,
		onOpenChange: props.onOpenChange
	});
	const context = {
		open: state.open,
		toggle: state.toggle,
		disabled: state.disabled,
		reducedMotion: () => props.reducedMotion ?? inheritedReducedMotion()
	};
	const rest = omit(props, "open", "defaultOpen", "disabled", "reducedMotion", "onOpenChange", "class", "children");
	return createComponent$1(CollapsibleContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				get "class"() {
					return mergeClasses("flex flex-col", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function CollapsibleTrigger(props) {
	const context = useCollapsible();
	const rest = omit(props, "children", "indicator", "class", "onClick");
	return createComponent$1(Button, mergeProps(rest, {
		unstyled: true,
		get disabled() {
			return context.disabled() || props.disabled;
		},
		get "aria-expanded"() {
			return context.open();
		},
		get "class"() {
			return mergeClasses("w-full min-h-7 flex items-center justify-between gap-3", props.class);
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented) context.toggle();
		},
		get children() {
			return [memo(() => {
				return props.children;
			}), memo(() => {
				return memo(() => {
					return props.indicator !== false;
				})() && createComponent$1(DisclosureIndicator, { get open() {
					return context.open;
				} });
			})];
		}
	}));
}
function CollapsibleContent(props) {
	const context = useCollapsible();
	const contentProps = omit(props, "children", "class", "style", "duration", "ease", "animateInitial");
	return createComponent$1(CollapsiblePresence, {
		get open() {
			return context.open();
		},
		get reducedMotion() {
			return context.reducedMotion();
		},
		get duration() {
			return props.duration;
		},
		get ease() {
			return props.ease;
		},
		get animateInitial() {
			return props.animateInitial;
		},
		get contentClass() {
			return props.class;
		},
		contentProps,
		get contentStyle() {
			return props.style;
		},
		get children() {
			return props.children;
		}
	});
}
function nextAccordionValue(current, type, item, collapsible = false) {
	return toggleSelection(current, item, type, collapsible) ?? "";
}
const AccordionContext = createContext();
const AccordionItemContext = createContext();
const useAccordion = () => {
	const value = useContext(AccordionContext);
	if (!value) throw new Error("Accordion parts must be inside Accordion");
	return value;
};
const useAccordionItem = () => {
	const value = useContext(AccordionItemContext);
	if (!value) throw new Error("Accordion parts must be inside AccordionItem");
	return value;
};
function Accordion(props) {
	const inheritedReducedMotion = useReducedMotion();
	const type = () => props.type ?? "single";
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? (type() === "multiple" ? [] : ""),
		disabled: () => props.disabled ?? false,
		onChange: props.onValueChange
	});
	const roving = createRovingFocus({ orientation: () => "vertical" });
	const context = {
		active: (item) => isSelected(state.value(), item),
		toggle: (item) => {
			state.set(nextAccordionValue(state.value(), type(), item, props.collapsible));
		},
		disabled: () => props.disabled ?? false,
		reducedMotion: () => props.reducedMotion ?? inheritedReducedMotion(),
		register: (value, node, disabled) => roving.register({
			id: value,
			target: node,
			disabled
		}),
		move: roving.move
	};
	const rest = omit(props, "type", "value", "defaultValue", "collapsible", "disabled", "reducedMotion", "onValueChange", "class", "children");
	return createComponent$1(AccordionContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				get "class"() {
					return mergeClasses("flex flex-col", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function AccordionItem(props) {
	const rest = omit(props, "value", "disabled", "children", "class");
	return createComponent$1(AccordionItemContext, {
		get value() {
			return {
				value: props.value,
				disabled: () => props.disabled ?? false
			};
		},
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				get "class"() {
					return mergeClasses("flex flex-col border-b border-subtle", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function AccordionTrigger(props) {
	const root = useAccordion();
	const item = useAccordionItem();
	const open = () => root.active(item.value);
	const rest = omit(props, "children", "class", "ref", "onClick", "onKeyDown");
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent$1(Button, mergeProps(rest, {
		unstyled: true,
		get disabled() {
			return root.disabled() || item.disabled() || props.disabled;
		},
		get "aria-expanded"() {
			return open();
		},
		get "class"() {
			return mergeClasses("w-full py-4 flex items-center justify-between gap-4", props.class);
		},
		ref: (node) => {
			unregister?.();
			unregister = root.register(item.value, node, () => root.disabled() || item.disabled() || (props.disabled ?? false));
			props.ref?.(node);
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented) root.toggle(item.value);
		},
		onKeyDown: (event) => {
			props.onKeyDown?.(event);
			if (!event.defaultPrevented && root.move(item.value, event.key)) event.preventDefault();
		},
		get children() {
			return [createComponent$1(Text, {
				class: "min-w-0 flex-1 whitespace-normal text-sm font-medium text-primary",
				get children() {
					return props.children;
				}
			}), createComponent$1(DisclosureIndicator, { open })];
		}
	}));
}
function AccordionContent(props) {
	const root = useAccordion();
	const item = useAccordionItem();
	const contentProps = omit(props, "children", "class", "style");
	return createComponent$1(CollapsiblePresence, {
		get open() {
			return root.active(item.value);
		},
		get reducedMotion() {
			return root.reducedMotion();
		},
		get contentClass() {
			return mergeClasses("pb-4", props.class);
		},
		contentProps,
		get contentStyle() {
			return props.style;
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
export { Accordion, AccordionContent, AccordionItem, AccordionTrigger, Collapsible, CollapsibleContent, CollapsibleTrigger, nextAccordionValue };

//# sourceMappingURL=disclosure.mjs.map