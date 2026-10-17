import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "./button.mjs";
import { ResponsiveGrid } from "./layout.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/prompt-suggestion.tsx
/** Container-responsive starter actions for empty conversations and assistants. */
function PromptSuggestions(props) {
	const rest = omit(props, "minColumnWidth", "maxColumns", "gap", "class");
	return createComponent$1(ResponsiveGrid, mergeProps(rest, {
		get minColumnWidth() {
			return props.minColumnWidth ?? 176;
		},
		get maxColumns() {
			return props.maxColumns ?? 3;
		},
		initialColumns: 1,
		get itemCount() {
			return props.itemCount;
		},
		balanceLastRow: true,
		get gap() {
			return props.gap ?? 8;
		},
		get "class"() {
			return mergeClasses("w-full", props.class);
		}
	}));
}
/** One explicit prompt choice; the application remains responsible for its payload. */
function PromptSuggestion(props) {
	const rest = omit(props, "title", "description", "icon", "class");
	return createComponent$1(Button, mergeProps(rest, {
		get "aria-label"() {
			return props["aria-label"] ?? props.title;
		},
		get variant() {
			return props.variant ?? "outline";
		},
		get "class"() {
			return mergeClasses("w-full h-auto min-h-14 min-w-0 items-start justify-start gap-2 border-subtle bg-transparent p-3 text-left shadow-none", props.class);
		},
		get children() {
			return [createComponent$1(Show, {
				get when() {
					return props.icon;
				},
				children: (source) => createComponent$1(Icon, {
					get source() {
						return source();
					},
					size: 15,
					class: "flex-none text-accent"
				})
			}), createComponent$1(View, {
				class: "min-w-0 flex-1 flex flex-col gap-1",
				get children() {
					return [createComponent$1(Text, {
						class: "whitespace-normal text-sm font-medium text-primary",
						get children() {
							return props.title;
						}
					}), createComponent$1(Show, {
						get when() {
							return props.description;
						},
						children: (description) => createComponent$1(Text, {
							class: "whitespace-normal text-xs text-secondary",
							get children() {
								return description();
							}
						})
					})];
				}
			})];
		}
	}));
}
//#endregion
export { PromptSuggestion, PromptSuggestions };

//# sourceMappingURL=prompt-suggestion.mjs.map