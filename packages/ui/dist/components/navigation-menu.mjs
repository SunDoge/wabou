import { Icon, View, rotate2d as rotate2d$1 } from "../primitives/view.mjs";
import { Button } from "./button.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { Popover } from "./popover.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, createContext, omit, onCleanup, useContext } from "solid-js";
import { applyRef, createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
//#region src/components/navigation-menu.tsx
const NavigationMenuContext = createContext();
const NavigationItemContext = createContext();
function requireNavigationMenu() {
	const context = useContext(NavigationMenuContext);
	if (!context) throw new Error("NavigationMenu child requires a root");
	return context;
}
function requireNavigationItem() {
	const context = useContext(NavigationItemContext);
	if (!context) throw new Error("NavigationMenu child requires an item");
	return context;
}
function NavigationMenu(props) {
	const entries = [];
	const contents = /* @__PURE__ */ new Map();
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? null,
		onChange: props.onValueChange
	});
	const enabled = () => entries.filter((entry) => !entry.disabled());
	const context = {
		openValue: state.value,
		setOpenValue: state.set,
		registerTrigger(entry) {
			entries.push(entry);
			return () => {
				const index = entries.indexOf(entry);
				if (index >= 0) entries.splice(index, 1);
			};
		},
		registerContent(value, render) {
			contents.set(value, render);
			return () => {
				if (contents.get(value) !== render) return;
				contents.delete(value);
			};
		},
		content() {
			const value = state.value();
			return value ? contents.get(value)?.() : void 0;
		},
		move(value, direction) {
			const candidates = enabled();
			if (candidates.length === 0) return false;
			const index = candidates.findIndex((entry) => entry.value === value);
			const target = direction === "next" ? candidates[index + 1] ?? (props.loop === false ? void 0 : candidates[0]) : candidates[index - 1] ?? (props.loop === false ? void 0 : candidates.at(-1));
			if (!target) return false;
			target.target.focus();
			if (state.value() !== null) state.set(target.value);
			return true;
		}
	};
	return createComponent(NavigationMenuContext, {
		value: context,
		get children() {
			return createComponent$1(Popover, {
				get "aria-label"() {
					return props["aria-label"];
				},
				get open() {
					return state.value() !== null;
				},
				onOpenChange: (open) => {
					if (!open) state.set(null);
				},
				placement: "bottom-start",
				outsidePointerStrategy: "passthrough",
				get contentClass() {
					return mergeClasses("w-[520px] max-w-full min-w-0 overflow-hidden p-2", props.viewportClass);
				},
				trigger: (trigger) => createComponent$1(View, {
					ref(r$) {
						var _ref$ = trigger.ref;
						typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : trigger.ref = r$;
					},
					role: "group",
					get "aria-label"() {
						return props["aria-label"];
					},
					get "class"() {
						return mergeClasses("relative inline-flex flex-none items-center justify-center", props.class);
					},
					get children() {
						return props.children;
					}
				}),
				get children() {
					return context.content();
				}
			});
		}
	});
}
function NavigationMenuList(props) {
	return createComponent$1(View, {
		role: "menubar",
		get "class"() {
			return mergeClasses("flex flex-none items-center justify-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function NavigationMenuItem(props) {
	const context = {
		value: props.value,
		disabled: () => props.disabled ?? false
	};
	return createComponent$1(NavigationItemContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				get "class"() {
					return mergeClasses("relative flex-none", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function navigationMenuTriggerClass(open, className) {
	return mergeClasses("h-8 px-3 inline-flex flex-none items-center justify-center gap-1 rounded-lg border-transparent text-sm font-medium", open ? "bg-selected text-primary" : "bg-transparent text-secondary", className);
}
function NavigationMenuTrigger(props) {
	const menu = requireNavigationMenu();
	const item = requireNavigationItem();
	let unregister;
	onCleanup(() => unregister?.());
	const open = () => menu.openValue() === item.value;
	const forwarded = omit(props, "class", "children", "ref", "onClick");
	return createComponent$1(Button, mergeProps(forwarded, {
		role: "menuitem",
		"aria-haspopup": "dialog",
		get "aria-expanded"() {
			return open();
		},
		get disabled() {
			return props.disabled ?? item.disabled();
		},
		ref: (node) => {
			unregister?.();
			unregister = menu.registerTrigger({
				value: item.value,
				target: node,
				disabled: () => props.disabled ?? item.disabled()
			});
			props.ref?.(node);
		},
		variant: "ghost",
		get "class"() {
			return navigationMenuTriggerClass(open(), props.class);
		},
		onPointerEnter: () => {
			if (menu.openValue() !== null) menu.setOpenValue(item.value);
		},
		onKeyDown: (event) => {
			if (event.key === "ArrowDown") {
				menu.setOpenValue(item.value);
				event.preventDefault();
			} else if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
				menu.move(item.value, event.key === "ArrowRight" ? "next" : "previous");
				event.preventDefault();
			}
			props.onKeyDown?.(event);
		},
		onClick: (event) => {
			menu.setOpenValue(open() ? null : item.value);
			props.onClick?.(event);
		},
		get children() {
			return [memo(() => {
				return props.children;
			}), createComponent$1(Icon, {
				"aria-hidden": "true",
				source: chevronDown,
				size: 14,
				class: "text-muted",
				get transform() {
					return memo(() => {
						return !!open();
					})() ? rotate2d$1(Math.PI) : void 0;
				}
			})];
		}
	}));
}
function NavigationMenuContent(props) {
	const menu = requireNavigationMenu();
	const item = requireNavigationItem();
	const forwarded = omit(props, "class", "children");
	const render = () => createComponent$1(View, mergeProps(forwarded, {
		role: "group",
		get "class"() {
			return mergeClasses("w-full min-w-0 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	}));
	const unregister = menu.registerContent(item.value, render);
	onCleanup(() => unregister?.());
}
function NavigationMenuLink(props) {
	const menu = requireNavigationMenu();
	const forwarded = omit(props, "active", "closeOnSelect", "class", "onClick");
	return createComponent$1(Button, mergeProps(forwarded, {
		role: "link",
		variant: "ghost",
		get "class"() {
			return mergeClasses("w-full h-auto min-w-0 flex flex-col items-start gap-1 rounded-lg p-2 text-left", props.active && "bg-selected", props.class);
		},
		onClick: (event) => {
			if (props.closeOnSelect ?? true) menu.setOpenValue(null);
			props.onClick?.(event);
		},
		get children() {
			return createComponent$1(View, {
				class: "w-full min-w-0 flex flex-col items-start gap-1",
				get children() {
					return props.children;
				}
			});
		}
	}));
}
function NavigationMenuIndicator(props) {
	return createComponent$1(View, mergeProps(props, {
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("absolute left-1/2 bottom-0 w-2 h-0.5 rounded-full bg-accent", props.class);
		}
	}));
}
/** The shared Popover content already is the native viewport. */
function NavigationMenuViewport() {}
//#endregion
export { NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem, NavigationMenuLink, NavigationMenuList, NavigationMenuTrigger, NavigationMenuViewport, navigationMenuTriggerClass };

//# sourceMappingURL=navigation-menu.mjs.map