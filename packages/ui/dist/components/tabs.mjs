import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { componentsDisabledInteractiveClass } from "./theme.mjs";
import { Button as Button$1 } from "./button.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { createRovingFocus } from "../primitives/interactions/roving-focus.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, createComponent, createContext, onCleanup, useContext } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import { match } from "ts-pattern";
import x from "lucide-static/icons/x.svg?raw";
//#region src/components/tabs.tsx
const orientationClass = (orientation, horizontal, vertical) => match(orientation).with("horizontal", () => horizontal).with("vertical", () => vertical).exhaustive();
const TabsContext = createContext();
function Tabs(props) {
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue,
		onChange: (value) => value !== void 0 && props.onValueChange?.(value)
	});
	const value = state.value;
	const select = (next) => {
		state.set(next);
	};
	const roving = createRovingFocus({
		orientation: () => props.orientation ?? "horizontal",
		preferred: (id) => value() === id,
		onMove: select
	});
	const context = {
		value,
		orientation: () => props.orientation ?? "horizontal",
		select,
		register: (next, node, disabled) => {
			const unregister = roving.register({
				id: next,
				target: node,
				disabled
			});
			if (value() === void 0 && !disabled()) select(next);
			return unregister;
		},
		activate: (next) => {
			roving.activate(next);
		},
		isTabStop: roving.isTabStop,
		move: roving.move
	};
	return createComponent(TabsContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				get "class"() {
					return mergeClasses("min-w-0 flex gap-3", orientationClass(context.orientation(), "flex-col", "flex-row"), props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function TabsList(props) {
	const context = useContext(TabsContext);
	if (!context) throw new Error("TabsList must be used inside Tabs");
	return createComponent$1(View, {
		role: "tablist",
		get "aria-label"() {
			return props["aria-label"];
		},
		get "aria-orientation"() {
			return context.orientation();
		},
		get "class"() {
			return memo(() => {
				return !!props.unstyled;
			})() ? props.class : mergeClasses("min-w-0 max-w-full flex-none flex items-center gap-1", orientationClass(context.orientation(), "flex-row overflow-x-scroll", "flex-col overflow-y-scroll"), match(props.variant ?? "default").with("default", () => "p-0.5 rounded-lg bg-control").with("line", () => "bg-transparent").exhaustive(), props.class);
		},
		get children() {
			return props.children;
		}
	});
}
/**
* Bounded, optionally closeable tab chrome. The tab trigger and close action
* remain sibling hit targets so closing a tab never selects it first.
*/
function TabsItem(props) {
	const context = useContext(TabsContext);
	if (!context) throw new Error("TabsItem must be used inside Tabs");
	const selected = () => context.value() === props.value;
	const itemClass = () => typeof props.class === "function" ? props.class({ selected: selected() }) : props.class;
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("h-8 min-w-24 max-w-56 flex flex-row items-center overflow-hidden rounded-md border border-transparent", selected() ? "bg-surface text-primary shadow-xs" : "bg-transparent text-muted", itemClass());
		},
		get children() {
			return [createComponent$1(TabsTrigger, {
				unstyled: true,
				get value() {
					return props.value;
				},
				get disabled() {
					return props.disabled;
				},
				class: (state) => mergeClasses("h-full min-w-0 flex-1 px-2 flex flex-row items-center gap-2", !selected() && state.hovered && "bg-control-hover text-primary", state.focusVisible && "border border-focus", typeof props.triggerClass === "function" ? props.triggerClass(state) : props.triggerClass),
				get children() {
					return props.children;
				}
			}), memo(() => {
				return memo(() => {
					return !!props.onClose;
				})() ? createComponent$1(Button$1, {
					variant: "ghost",
					size: "icon",
					get "aria-label"() {
						return props.closeLabel ?? `Close ${props.value}`;
					},
					class: "w-6 h-6 flex-none text-muted",
					style: { padding: 0 },
					onClick: (event) => {
						event.stopPropagation();
						props.onClose?.();
					},
					get children() {
						return createComponent$1(Icon, {
							source: x,
							size: 12
						});
					}
				}) : null;
			})];
		}
	});
}
function TabsTrigger(props) {
	const context = useContext(TabsContext);
	if (!context) throw new Error("TabsTrigger must be used inside Tabs");
	const selected = () => context.value() === props.value;
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent$1(Button, {
		unstyled: true,
		role: "tab",
		get "aria-label"() {
			return props["aria-label"];
		},
		get disabled() {
			return props.disabled;
		},
		get selected() {
			return selected();
		},
		get "aria-selected"() {
			return selected();
		},
		get focusOrder() {
			return context.isTabStop(props.value) ? 0 : -1;
		},
		ref: (node) => {
			unregister?.();
			unregister = context.register(props.value, node, () => props.disabled ?? false);
		},
		class: (state) => props.unstyled ? typeof props.class === "function" ? props.class(state) : props.class ?? "" : mergeClasses("h-7 px-3 flex-none whitespace-nowrap items-center justify-center rounded-md border border-transparent text-sm font-medium", match({
			selected: selected(),
			hovered: !state.disabled && state.hovered
		}).with({ selected: true }, () => "bg-surface text-primary shadow-xs").with({ hovered: true }, () => "bg-control-hover text-primary").otherwise(() => "bg-transparent text-muted"), state.focusVisible && "border-focus", typeof props.class === "function" ? props.class(state) : props.class, componentsDisabledInteractiveClass(state.disabled)),
		onClick: () => context.select(props.value),
		onFocus: () => context.activate(props.value),
		onKeyDown: (event) => {
			if (context.move(props.value, event.key)) event.preventDefault();
		},
		get children() {
			return memo(() => {
				return !!props.unstyled;
			})() ? props.children : createComponent$1(Text, {
				class: "text-sm font-medium",
				get children() {
					return props.children;
				}
			});
		}
	});
}
function TabsContent(props) {
	const context = useContext(TabsContext);
	if (!context) throw new Error("TabsContent must be used inside Tabs");
	const selected = () => context.value() === props.value;
	return createComponent$1(Show, {
		get when() {
			return props.keepMounted || selected();
		},
		get children() {
			return createComponent$1(View, {
				role: "tabpanel",
				get "aria-hidden"() {
					return !selected();
				},
				get "class"() {
					return mergeClasses("min-w-0 flex flex-col", orientationClass(context.orientation(), "w-full flex-none", "flex-1"), props.class);
				},
				get style() {
					return { display: selected() ? "flex" : "none" };
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
export { Tabs, TabsContent, TabsItem, TabsList, TabsTrigger };

//# sourceMappingURL=tabs.mjs.map