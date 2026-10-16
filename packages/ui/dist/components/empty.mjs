import { Text, View } from "../primitives/view.mjs";
import { Column } from "../primitives/layout.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/empty.tsx
function emptyClass(variant = "surface", className) {
	return mergeClasses("w-full min-w-0 flex-1 p-8 items-center justify-center gap-6 text-center", variant === "surface" ? "min-h-64 rounded-xl border border-subtle bg-surface shadow-xs" : "min-h-0 bg-transparent", className);
}
/** A composable empty-state region based on shadcn's Empty anatomy. */
function Empty(props) {
	const rest = omit(props, "variant", "class", "children");
	return createComponent$1(Column, mergeProps(rest, {
		get "class"() {
			return emptyClass(props.variant, props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function EmptyHeader(props) {
	return createComponent$1(Column, mergeProps(props, {
		get "class"() {
			return mergeClasses("w-full max-w-sm min-w-0 items-center gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function emptyMediaClass(variant = "default", className) {
	return mergeClasses("mb-2 flex-none flex items-center justify-center", match(variant).with("default", () => "bg-transparent").with("icon", () => mergeClasses("w-10 h-10 rounded-lg", "bg-control text-primary")).exhaustive(), className);
}
function EmptyMedia(props) {
	const rest = omit(props, "variant", "class", "children");
	return createComponent$1(View, mergeProps(rest, {
		get "class"() {
			return emptyMediaClass(props.variant, props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function EmptyTitle(props) {
	return createComponent$1(Text, mergeProps(props, {
		get role() {
			return props.role ?? "heading";
		},
		get "class"() {
			return mergeClasses("text-lg font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function EmptyDescription(props) {
	return createComponent$1(Text, mergeProps(props, {
		get maxLines() {
			return props.maxLines ?? 2;
		},
		get "class"() {
			return mergeClasses("w-full min-h-10 min-w-0 whitespace-normal text-center text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function EmptyContent(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("w-full max-w-sm min-w-0 flex flex-col items-center gap-4 text-sm", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle, emptyClass, emptyMediaClass };

//# sourceMappingURL=empty.mjs.map