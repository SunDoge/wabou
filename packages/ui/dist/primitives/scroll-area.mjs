import { View } from "./view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { applyRef, createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/primitives/scroll-area.tsx
const scrollAreaViewportClass = (className) => mergeClasses("min-w-0 min-h-0 overflow-x-hidden overflow-y-auto", className);
/**
* Vertical native scroll viewport with explicit sizing.
*
* The inner wrapper deliberately cannot shrink. This makes its intrinsic
* height become the viewport's scroll extent instead of allowing a flex
* parent to compress overflowing sections until no scroll range remains.
* The viewport also locks its cross axis. Otherwise focus reveal can move a
* nominally vertical viewport sideways when a descendant is slightly wider,
* making split-pane edges appear clipped. It deliberately does not grow:
* implicit `flex-1` makes a
* nested scroll area expand with an ancestor's intrinsic content instead of
* establishing its own scroll range.
*/
function ScrollArea(props) {
	const forwarded = omit(props, "children", "class", "contentClass", "style", "ref", "scrollbar", "onScroll");
	return createComponent$1(View, mergeProps(forwarded, {
		ref(r$) {
			var _ref$ = props.ref;
			typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : props.ref = r$;
		},
		get "class"() {
			return scrollAreaViewportClass(props.class);
		},
		get style() {
			return props.style;
		},
		get scrollbar() {
			return props.scrollbar;
		},
		get onScroll() {
			return props.onScroll;
		},
		get children() {
			return createComponent$1(View, {
				get "class"() {
					return mergeClasses("flex-none flex flex-col min-h-full", props.contentClass);
				},
				get children() {
					return props.children;
				}
			});
		}
	}));
}
//#endregion
export { ScrollArea, scrollAreaViewportClass };

//# sourceMappingURL=scroll-area.mjs.map