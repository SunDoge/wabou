import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { Button as Button$1 } from "./button.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { clampPage, createPaginationRange, normalizePageCount } from "./pagination-state.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, createComponent, createContext, omit, useContext } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import ellipsis from "lucide-static/icons/ellipsis.svg?raw";
//#region src/components/navigation.tsx
function Breadcrumb(props) {
	const rest = omit(props, "class", "children");
	return createComponent$1(View, mergeProps(rest, {
		role: "group",
		get "aria-label"() {
			return props["aria-label"] ?? "Breadcrumb";
		},
		get "class"() {
			return mergeClasses("min-w-0", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function BreadcrumbList(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("min-w-0 flex flex-wrap items-center gap-1.5 text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function BreadcrumbItem(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("min-w-0 flex items-center gap-1.5", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function BreadcrumbLink(props) {
	return createComponent$1(Button, mergeProps(props, {
		unstyled: true,
		role: "link",
		class: (state) => mergeClasses("min-w-0 rounded-sm text-sm text-secondary", state.hovered && "text-primary", state.focusVisible && "border border-focus", props.class)
	}));
}
function BreadcrumbPage(props) {
	const rest = omit(props, "class", "children");
	return createComponent$1(Text, mergeProps(rest, {
		role: "link",
		"aria-disabled": "true",
		"aria-current": "page",
		get "class"() {
			return mergeClasses("min-w-0 text-sm font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function BreadcrumbSeparator(props) {
	const rest = omit(props, "class", "children");
	return createComponent$1(View, mergeProps(rest, {
		role: "presentation",
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("w-4 h-4 flex-none flex items-center justify-center text-muted", props.class);
		},
		get children() {
			return props.children ?? createComponent$1(Icon, {
				source: chevronRight,
				size: 14
			});
		}
	}));
}
function BreadcrumbEllipsis(props) {
	const rest = omit(props, "class");
	return createComponent$1(View, mergeProps(rest, {
		role: "presentation",
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("w-8 h-8 flex-none flex items-center justify-center text-muted", props.class);
		},
		get children() {
			return createComponent$1(Icon, {
				source: ellipsis,
				size: 16
			});
		}
	}));
}
const PaginationContext = createContext();
function usePaginationContext() {
	const context = useContext(PaginationContext);
	if (!context) throw new Error("Pagination controls must be used inside Pagination");
	return context;
}
function Pagination(props) {
	const state = createControllableState({
		value: () => props.page,
		defaultValue: props.defaultPage ?? 1,
		onChange: props.onPageChange
	});
	const count = () => normalizePageCount(props.count);
	const page = () => clampPage(state.value(), count());
	const context = {
		count,
		page,
		disabled: () => props.disabled ?? false,
		select: (next) => {
			if (context.disabled()) return;
			state.set(clampPage(next, count()));
		}
	};
	const content = () => createComponent$1(View, {
		role: "group",
		get "aria-label"() {
			return props["aria-label"] ?? "Pagination";
		},
		get "aria-disabled"() {
			return context.disabled() || void 0;
		},
		get "class"() {
			return mergeClasses("flex items-center", props.class);
		},
		get children() {
			return props.children;
		}
	});
	return createComponent(PaginationContext, {
		value: context,
		get children() {
			return content();
		}
	});
}
function PaginationContent(props) {
	return createComponent$1(View, mergeProps(props, { get "class"() {
		return mergeClasses("flex items-center gap-1", props.class);
	} }));
}
function PaginationItem(props) {
	return createComponent$1(View, mergeProps(props, { get "class"() {
		return mergeClasses("flex-none", props.class);
	} }));
}
function PaginationLink(props) {
	const context = usePaginationContext();
	const forwarded = omit(props, "active", "page");
	const active = () => props.active ?? (props.page !== void 0 && context.page() === props.page);
	return createComponent$1(Button$1, mergeProps(forwarded, {
		role: "link",
		size: "icon",
		get variant() {
			return active() ? "outline" : "ghost";
		},
		get selected() {
			return active();
		},
		get "aria-current"() {
			return active() ? "page" : void 0;
		},
		get "aria-label"() {
			return props["aria-label"] ?? (props.page === void 0 ? void 0 : `Page ${props.page}`);
		},
		get disabled() {
			return props.disabled ?? context.disabled();
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented && props.page !== void 0) context.select(props.page);
		}
	}));
}
function PaginationEllipsis(props) {
	return createComponent$1(Text, {
		"aria-hidden": true,
		get "class"() {
			return mergeClasses("w-8 text-center text-muted", props.class);
		},
		children: "..."
	});
}
function PaginationItems(props) {
	const context = usePaginationContext();
	const items = () => createPaginationRange({
		count: context.count(),
		page: context.page(),
		siblingCount: props.siblingCount,
		boundaryCount: props.boundaryCount
	});
	return createComponent$1(For, {
		get each() {
			return items();
		},
		children: (item) => typeof item === "number" ? props.renderItem?.(item) ?? createComponent$1(PaginationItem, { get children() {
			return createComponent$1(PaginationLink, {
				page: item,
				get children() {
					return String(item);
				}
			});
		} }) : props.renderEllipsis?.(item === "ellipsis-start" ? "start" : "end") ?? createComponent$1(PaginationEllipsis, {})
	});
}
function PaginationPrevious(props) {
	const context = usePaginationContext();
	return createComponent$1(Button$1, mergeProps(props, {
		variant: "ghost",
		size: "sm",
		get disabled() {
			return props.disabled ?? (context.disabled() || context.page() <= 1);
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented) context.select(context.page() - 1);
		},
		get children() {
			return props.children ?? "Previous";
		}
	}));
}
function PaginationNext(props) {
	const context = usePaginationContext();
	return createComponent$1(Button$1, mergeProps(props, {
		variant: "ghost",
		size: "sm",
		get disabled() {
			return props.disabled ?? (context.disabled() || context.page() >= context.count());
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented) context.select(context.page() + 1);
		},
		get children() {
			return props.children ?? "Next";
		}
	}));
}
//#endregion
export { Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator, Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationItems, PaginationLink, PaginationNext, PaginationPrevious, clampPage, createPaginationRange, normalizePageCount };

//# sourceMappingURL=navigation.mjs.map