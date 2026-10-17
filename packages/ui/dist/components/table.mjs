import { createHover } from "../primitives/hover.mjs";
import { Text, View } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/table.tsx
/**
* A horizontally scrollable table surface.
*
* Wabou has no implicit HTML table layout. Columns align because every row
* uses the same flex-cell anatomy; applications can override individual cell
* widths with the usual flex and width utilities.
*/
function Table(props) {
	const rest = omit(props, "class", "contentClass", "children");
	return createComponent$1(View, mergeProps(rest, {
		role: "table",
		get "class"() {
			return mergeClasses("relative w-full min-w-0 overflow-x-auto overflow-y-hidden", props.class);
		},
		get children() {
			return createComponent$1(View, {
				get "class"() {
					return mergeClasses("w-full min-w-full flex-none flex flex-col text-sm", props.contentClass);
				},
				get children() {
					return props.children;
				}
			});
		}
	}));
}
function TableHeader(props) {
	return createComponent$1(View, mergeProps(props, {
		role: "group",
		get "aria-label"() {
			return props["aria-label"] ?? "Table header";
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex-none flex flex-col", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableBody(props) {
	return createComponent$1(View, mergeProps(props, {
		role: "group",
		get "aria-label"() {
			return props["aria-label"] ?? "Table body";
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex-none flex flex-col", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableFooter(props) {
	return createComponent$1(View, mergeProps(props, {
		role: "group",
		get "aria-label"() {
			return props["aria-label"] ?? "Table footer";
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex-none flex flex-col border-t border-subtle bg-control", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableRow(props) {
	const hover = createHover();
	const rest = omit(props, "class", "selected", "children", "onPointerEnter", "onPointerLeave");
	return createComponent$1(View, mergeProps(rest, {
		role: "row",
		get "aria-selected"() {
			return props.selected;
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 min-h-11 flex-none flex flex-row items-stretch border-b border-subtle", props.selected ? "bg-selected" : "bg-surface", hover.hovered() && !props.selected ? "bg-control-hover" : void 0, props.class);
		},
		onPointerEnter: (event) => {
			hover.bindings.onPointerEnter();
			props.onPointerEnter?.(event);
		},
		onPointerLeave: (event) => {
			hover.bindings.onPointerLeave();
			props.onPointerLeave?.(event);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableHead(props) {
	return createComponent$1(Text, mergeProps(props, {
		role: "columnheader",
		get "class"() {
			return mergeClasses("min-w-32 flex-1 px-3 flex items-center whitespace-nowrap text-xs font-medium text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableCell(props) {
	return createComponent$1(View, mergeProps(props, {
		role: "cell",
		get "class"() {
			return mergeClasses("min-w-32 flex-1 px-3 flex items-center text-sm text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableCaption(props) {
	return createComponent$1(Text, mergeProps(props, {
		get role() {
			return props.role ?? "label";
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex-none px-3 py-3 whitespace-normal text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow };

//# sourceMappingURL=table.mjs.map