import { View } from "../primitives/view.mjs";
import { useComponentsTheme } from "./theme.mjs";
import { useWindow } from "@wabou/core";
import { mergeClasses, rgba as rgba$1, shadow } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/title-bar.tsx
function windowFrameMaterialStyle(theme, material = "solid") {
	if (material === "solid") return {};
	return { "background-color": rgba$1(theme === "dark" ? 286795752 : 4177198312) };
}
function windowFrameBackdropClassList(maximized, rounded = true) {
	return { "p-3": rounded && !maximized };
}
function windowFrameClientClassList(maximized, rounded = true, classList = {}) {
	const decorated = rounded && !maximized;
	return {
		...classList,
		"rounded-xl": decorated,
		border: decorated,
		"border-subtle": decorated,
		"overflow-hidden": decorated
	};
}
/** Two restrained client-decoration layers sized to fit the 12px backdrop. */
function windowFrameShadows(theme) {
	const ambient = theme === "dark" ? 77 : 36;
	const contact = theme === "dark" ? 82 : 46;
	return [shadow({
		offsetY: 2,
		spread: -1,
		stdDev: 3,
		color: ambient,
		radius: 12
	}), shadow({
		offsetY: 1,
		spread: 0,
		stdDev: 1.5,
		color: contact,
		radius: 12
	})];
}
/**
* Root frame for an application-owned title bar and window chrome.
*
* Rounded outer corners require the native window to preserve alpha and the
* Rust host to clear with a transparent base color. Maximized windows are
* intentionally square so their content reaches every display edge.
*/
function WindowFrame(props) {
	const viewProps = omit(props, "rounded", "material");
	const window = useWindow();
	const theme = useComponentsTheme();
	const decorated = () => props.rounded !== false && !window.maximized();
	return createComponent$1(View, {
		class: "w-full h-full bg-transparent",
		get classList() {
			return windowFrameBackdropClassList(window.maximized(), props.rounded !== false);
		},
		get children() {
			return createComponent$1(View, mergeProps(viewProps, {
				get "class"() {
					return mergeClasses("w-full h-full", viewProps.class);
				},
				get classList() {
					return windowFrameClientClassList(window.maximized(), props.rounded !== false, viewProps.classList);
				},
				get style() {
					return {
						...windowFrameMaterialStyle(theme(), props.material),
						...viewProps.style
					};
				},
				get shadows() {
					return memo(() => {
						return viewProps.shadows !== void 0;
					})() ? viewProps.shadows : memo(() => {
						return !!decorated();
					})() ? windowFrameShadows(theme()) : null;
				}
			}));
		}
	});
}
const titleBarClass = "border-b border-subtle";
const titleBarLayoutStyle = {
	display: "flex",
	"flex-direction": "row",
	"align-items": "center",
	"flex-shrink": 0,
	height: "40px"
};
const titleBarDragRegionLayoutStyle = {
	display: "flex",
	"flex-direction": "row",
	"align-items": "center",
	"flex-grow": 1,
	"flex-shrink": 1,
	"flex-basis": "0%",
	height: "100%"
};
/** Layout shell for an application-owned title bar. */
function TitleBar(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses(titleBarClass, props.class);
		},
		get style() {
			return {
				...titleBarLayoutStyle,
				...props.style
			};
		}
	}));
}
/** Explicit non-interactive region that moves the native window. */
function TitleBarDragRegion(props) {
	const window = useWindow();
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return props.class;
		},
		get style() {
			return {
				...titleBarDragRegionLayoutStyle,
				...props.style
			};
		},
		onPointerDown: (event) => {
			if (event.button === 0) window.startDragging();
		},
		onDblClick: () => window.setMaximized(!window.maximized())
	}));
}
//#endregion
export { TitleBar, TitleBarDragRegion, WindowFrame, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, windowFrameBackdropClassList, windowFrameClientClassList, windowFrameMaterialStyle, windowFrameShadows };

//# sourceMappingURL=title-bar.mjs.map