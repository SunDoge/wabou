import { Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { ScrollArea } from "../primitives/scroll-area.mjs";
import { componentsElevation, useComponentsTheme } from "./theme.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { SearchField } from "./search-field.mjs";
import { workbenchHeaderClass } from "./workbench-style.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, createContext, omit, useContext } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/sidebar.tsx
const SidebarMenuContext = createContext({
	managed: false,
	value: () => void 0,
	select: () => {}
});
/**
* Filter grouped sidebar data without taking ownership of routing or identity.
* Group labels participate in matching so a query can reveal a whole section.
*/
function filterSidebarGroups(groups, query, searchableText) {
	const needle = query.trim().toLowerCase();
	if (!needle) return groups.map((group) => ({
		...group,
		items: [...group.items]
	}));
	return groups.flatMap((group) => {
		const items = group.label.toLowerCase().includes(needle) ? [...group.items] : group.items.filter((item) => searchableText(item).toLowerCase().includes(needle));
		return items.length === 0 ? [] : [{
			...group,
			items
		}];
	});
}
/** Structural application sidebar. State, routing and width remain explicit. */
function Sidebar(props) {
	const theme = useComponentsTheme();
	const forwarded = omit(props, "class", "elevation", "shadows");
	return createComponent$1(View, mergeProps(forwarded, {
		get role() {
			return props.role ?? "group";
		},
		get "class"() {
			return mergeClasses("h-full min-h-0 flex-none flex flex-col overflow-hidden bg-surface-muted", props.class);
		},
		get shadows() {
			return memo(() => {
				return !!(props.shadows === void 0 && props.elevation);
			})() ? componentsElevation(theme(), props.elevation) : props.shadows;
		}
	}));
}
function SidebarHeader(props) {
	return createComponent$1(View, mergeProps(props, { get "class"() {
		return workbenchHeaderClass(props.class);
	} }));
}
function SidebarSearch(props) {
	const forwarded = omit(props, "class");
	const quiet = () => props.variant === "quiet";
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("flex-none p-2", quiet() ? "bg-surface-muted" : "border-b border-subtle bg-surface");
		},
		get children() {
			return createComponent$1(SearchField, mergeProps(forwarded, {
				get placeholder() {
					return props.placeholder ?? "Search";
				},
				get "class"() {
					return mergeClasses("w-full", props.class);
				}
			}));
		}
	});
}
/** The only scrolling region in a standard sidebar; header/footer stay fixed. */
function SidebarContent(props) {
	return createComponent$1(ScrollArea, mergeProps(props, {
		get "class"() {
			return mergeClasses("min-h-0 flex-1", props.class);
		},
		get contentClass() {
			return mergeClasses("px-3 py-3", props.contentClass);
		}
	}));
}
function SidebarGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "group";
		},
		get "class"() {
			return mergeClasses("flex-none flex flex-col gap-2 mb-3", props.class);
		}
	}));
}
function SidebarGroupLabel(props) {
	return createComponent$1(Text, mergeProps(props, { get "class"() {
		return mergeClasses("h-8 px-2 flex-none flex items-center text-xs font-medium text-muted", props.class);
	} }));
}
/**
* Single-selection navigation scope for sidebar destinations.
* Buttons without a value remain actions and never become selected items.
*/
function SidebarMenu(props) {
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue,
		onChange: (value) => value !== void 0 && props.onValueChange?.(value)
	});
	const forwarded = omit(props, "value", "defaultValue", "onValueChange", "class", "children");
	return createComponent(SidebarMenuContext, {
		value: {
			managed: true,
			value: state.value,
			select: (value) => state.set(value)
		},
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				get role() {
					return props.role ?? "group";
				},
				get "class"() {
					return mergeClasses("min-w-0 flex flex-col gap-2", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
/** Consistent navigation row; applications still own activation and routing. */
function SidebarMenuButton(props) {
	const menu = useContext(SidebarMenuContext);
	const forwarded = omit(props, "class", "value", "selected", "onClick");
	const selected = () => props.value !== void 0 && menu.managed ? menu.value() === props.value : props.selected ?? false;
	return createComponent$1(Button, mergeProps(forwarded, {
		unstyled: true,
		get selected() {
			return selected();
		},
		get "aria-selected"() {
			return selected();
		},
		class: (state) => mergeClasses("w-full min-w-0 h-8 px-2 justify-start gap-2 rounded-md border border-transparent text-sm", state.pressed ? "bg-control-pressed text-primary" : state.selected ? "bg-selected font-medium text-primary" : state.hovered ? "bg-control-hover text-primary" : "bg-transparent text-secondary", state.focusVisible && "border-focus", props.class),
		onClick: (event) => {
			if (props.value !== void 0 && menu.managed) menu.select(props.value);
			props.onClick?.(event);
		}
	}));
}
/** Fixed icon slot that keeps navigation labels on one shared baseline. */
function SidebarMenuIcon(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "img";
		},
		get "class"() {
			return mergeClasses("w-4 h-4 flex-none flex items-center justify-center", props.class);
		}
	}));
}
/** Truncating label slot for rows that also contain icons or suffix actions. */
function SidebarMenuLabel(props) {
	return createComponent$1(Text, mergeProps(props, { get "class"() {
		return mergeClasses("min-w-0 flex-1 truncate text-left", props.class);
	} }));
}
/** End-aligned metadata or action slot that never compresses the row label. */
function SidebarMenuSuffix(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "group";
		},
		get "class"() {
			return mergeClasses("min-w-0 flex-none flex items-center justify-end", props.class);
		}
	}));
}
function SidebarEmpty(props) {
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("px-3 py-6 flex flex-col items-center gap-1", props.class);
		},
		get children() {
			return [createComponent$1(Text, {
				role: "status",
				class: "text-sm text-secondary",
				get children() {
					return props.title ?? "No results found";
				}
			}), memo(() => {
				return memo(() => {
					return !!props.description;
				})() ? createComponent$1(Text, {
					class: "text-xs text-muted",
					get children() {
						return props.description;
					}
				}) : null;
			})];
		}
	});
}
function SidebarFooter(props) {
	return createComponent$1(View, mergeProps(props, { get "class"() {
		return mergeClasses("flex-none border-t border-subtle bg-surface", props.class);
	} }));
}
//#endregion
export { Sidebar, SidebarContent, SidebarEmpty, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuIcon, SidebarMenuLabel, SidebarMenuSuffix, SidebarSearch, filterSidebarGroups };

//# sourceMappingURL=sidebar.mjs.map