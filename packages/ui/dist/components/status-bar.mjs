import { Text, View } from "../primitives/view.mjs";
import { componentToneClass } from "./tone.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/status-bar.tsx
/** Persistent low-emphasis application state at the bottom of a desktop window. */
function StatusBar(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "status";
		},
		get "class"() {
			return mergeClasses("h-7 w-full min-w-0 flex-none flex flex-row items-center gap-1 px-2 border-t border-subtle bg-control", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function StatusBarItem(props) {
	const forwarded = omit(props, "class", "grow", "children");
	return createComponent$1(Text, mergeProps(forwarded, {
		get "class"() {
			return mergeClasses("h-full min-w-0 px-1.5 flex items-center text-xs text-muted whitespace-nowrap", props.grow ? "flex-1" : "flex-none", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function StatusBarSeparator(props) {
	return createComponent$1(View, {
		role: "separator",
		"aria-orientation": "vertical",
		get "class"() {
			return mergeClasses("w-px h-4 flex-none bg-subtle", props.class);
		}
	});
}
/** A shrink-safe status bar group for icons, indicators and related text. */
function StatusBarGroup(props) {
	const forwarded = omit(props, "class", "grow", "shrink", "children");
	return createComponent$1(View, mergeProps(forwarded, {
		get "class"() {
			return mergeClasses("h-full min-w-0 px-1.5 flex flex-row items-center gap-1.5 text-xs text-muted", props.grow ? "flex-1" : props.shrink ? "shrink" : "flex-none", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
/** Compact, decorative state indicator with a theme-aware semantic tone. */
function StatusBarIndicator(props) {
	return createComponent$1(View, {
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("w-1.5 h-1.5 flex-none rounded-full", componentToneClass(props.tone === "muted" ? "neutral" : props.tone ?? "neutral", "indicator"), props.class);
		}
	});
}
//#endregion
export { StatusBar, StatusBarGroup, StatusBarIndicator, StatusBarItem, StatusBarSeparator };

//# sourceMappingURL=status-bar.mjs.map