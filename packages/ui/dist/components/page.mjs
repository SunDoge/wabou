import { Text, View } from "../primitives/view.mjs";
import { ScrollArea } from "../primitives/scroll-area.mjs";
import { createScrollReset } from "../primitives/scroll-reset.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/page.tsx
const pageViewportClass = (className) => mergeClasses("min-w-0 min-h-0 flex-1", className);
const pageViewportContentClass = (className) => mergeClasses("w-full", className);
/**
* A full-height application page boundary.
*
* This composes native scrolling with an explicitly sized content wrapper and
* optional identity-based scroll reset. Page implementations can therefore
* focus on their own layout instead of reconstructing flex/overflow rules.
*/
function PageViewport(props) {
	let viewport;
	const forwarded = omit(props, "children", "class", "contentClass", "resetKey", "ref", "style", "scrollbar", "onScroll");
	createScrollReset({
		target: () => viewport,
		key: () => props.resetKey
	});
	return createComponent$1(ScrollArea, mergeProps(forwarded, {
		get "class"() {
			return pageViewportClass(props.class);
		},
		get contentClass() {
			return pageViewportContentClass(props.contentClass);
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
		ref: (node) => {
			viewport = node;
			props.ref?.(node);
		},
		get children() {
			return props.children;
		}
	}));
}
const pageHeaderClass = (className, stacked = false) => mergeClasses("min-w-0 min-h-12 flex-none flex justify-between gap-4", stacked ? "flex-col items-stretch" : "flex-row items-center", className);
const pageHeaderTitleClass = () => "whitespace-nowrap text-2xl font-semibold text-primary";
const pageHeaderDescriptionClass = (stacked = false) => mergeClasses("text-sm text-secondary", stacked ? "whitespace-normal" : "truncate");
/** Consistent page title, supporting text and trailing application actions. */
function PageHeader(props) {
	return createComponent$1(View, {
		get "class"() {
			return pageHeaderClass(props.class, props.stacked);
		},
		get children() {
			return [createComponent$1(View, {
				class: "min-w-0 flex-1 flex flex-row items-center gap-3",
				get children() {
					return [createComponent$1(View, {
						class: "min-w-0 flex flex-col gap-1",
						get children() {
							return [createComponent$1(Text, {
								role: "heading",
								get "class"() {
									return pageHeaderTitleClass();
								},
								get children() {
									return props.title;
								}
							}), createComponent$1(Show, {
								get when() {
									return props.description;
								},
								children: (description) => createComponent$1(Text, {
									get "class"() {
										return pageHeaderDescriptionClass(props.stacked);
									},
									get children() {
										return description();
									}
								})
							})];
						}
					}), memo(() => {
						return props.titleAdornment;
					})];
				}
			}), createComponent$1(Show, {
				get when() {
					return props.actions;
				},
				children: (actions) => createComponent$1(View, {
					class: "flex-none flex flex-row items-center gap-2",
					get classList() {
						return { "w-full": props.stacked };
					},
					get children() {
						return actions();
					}
				})
			})];
		}
	});
}
//#endregion
export { PageHeader, PageViewport, pageHeaderClass, pageHeaderDescriptionClass, pageHeaderTitleClass, pageViewportClass, pageViewportContentClass };

//# sourceMappingURL=page.mjs.map