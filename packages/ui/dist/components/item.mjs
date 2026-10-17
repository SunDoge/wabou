import { Text, View } from "../primitives/view.mjs";
import { componentsDisabledInteractiveClass } from "./theme.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/item.tsx
function itemClass(variant = "default", size = "default", className) {
	const colors = match(variant).with("default", () => "border-transparent bg-transparent").with("outline", () => "border-subtle bg-transparent").with("muted", () => "border-transparent bg-control").exhaustive();
	const spacing = match(size).with("default", () => "gap-4 p-4").with("sm", () => "gap-2 px-4 py-3").exhaustive();
	return mergeClasses("w-full min-w-0 flex flex-row flex-wrap items-center rounded-lg border text-sm", colors, spacing, className);
}
/** A composable list row based on shadcn's Item anatomy. */
function Item(props) {
	const rest = omit(props, "variant", "size", "selected", "disabled", "class", "style", "children");
	return createComponent$1(View, mergeProps(rest, {
		get role() {
			return props.role ?? "listitem";
		},
		get "aria-selected"() {
			return props.selected;
		},
		get interactionBlocked() {
			return props.disabled || props.interactionBlocked;
		},
		get "class"() {
			return itemClass(props.variant, props.size, mergeClasses(props.selected && "border-strong bg-selected", props.class, componentsDisabledInteractiveClass(props.disabled ?? false)));
		},
		get style() {
			return props.style;
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "list";
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex flex-col", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemSeparator(props) {
	return createComponent$1(View, {
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("w-full h-px flex-none bg-subtle", props.class);
		}
	});
}
function itemMediaClass(variant = "default", className) {
	return mergeClasses("flex-none flex items-center justify-center gap-2", match(variant).with("default", () => "bg-transparent").with("icon", () => "w-8 h-8 rounded-sm border border-subtle bg-control").with("image", () => "w-10 h-10 overflow-hidden rounded-sm").exhaustive(), className);
}
function ItemMedia(props) {
	return createComponent$1(View, {
		get "class"() {
			return itemMediaClass(props.variant, props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function ItemContent(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("flex-1 min-w-0 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemTitle(props) {
	return createComponent$1(Text, mergeProps(props, {
		get maxLines() {
			return props.maxLines ?? 1;
		},
		get "class"() {
			return mergeClasses("min-w-0 truncate text-sm font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemDescription(props) {
	return createComponent$1(Text, mergeProps(props, {
		get maxLines() {
			return props.maxLines ?? 2;
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 whitespace-normal text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemActions(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("flex-none flex items-center gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemHeader(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("w-full min-w-0 flex items-center justify-between gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
const ItemFooter = ItemHeader;
//#endregion
export { Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia, ItemSeparator, ItemTitle, itemClass, itemMediaClass };

//# sourceMappingURL=item.mjs.map