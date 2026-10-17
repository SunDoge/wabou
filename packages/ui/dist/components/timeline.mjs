import { Text, View } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
//#region src/components/timeline.tsx
function Timeline(props) {
	return createComponent$1(View, {
		role: "group",
		get "aria-label"() {
			return props["aria-label"] ?? "Timeline";
		},
		get "class"() {
			return mergeClasses("min-w-0 flex flex-col", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return props.items;
				},
				children: (item, index) => createComponent$1(View, {
					class: "min-w-0 flex flex-row gap-3",
					get children() {
						return [createComponent$1(View, {
							class: "w-4 flex-none flex flex-col items-center",
							get children() {
								return [createComponent$1(View, { get "class"() {
									return mergeClasses("w-3 h-3 flex-none rounded-full border", item.status === "complete" ? "border-accent bg-accent" : item.status === "current" ? "border-accent bg-surface" : "border-subtle bg-control");
								} }), memo(() => {
									return memo(() => {
										return index() < props.items.length - 1;
									})() && createComponent$1(View, { class: "w-px min-h-8 flex-1 bg-subtle" });
								})];
							}
						}), createComponent$1(View, {
							class: "min-w-0 flex-1 pb-5",
							get children() {
								return [createComponent$1(View, {
									class: "flex flex-row justify-between gap-3",
									get children() {
										return [createComponent$1(Text, {
											class: "min-w-0 font-medium text-primary",
											get children() {
												return item.title;
											}
										}), memo(() => {
											return memo(() => {
												return !!item.time;
											})() ? createComponent$1(Text, {
												class: "flex-none text-xs text-muted",
												get children() {
													return item.time;
												}
											}) : item.time;
										})];
									}
								}), memo(() => {
									return memo(() => {
										return !!item.description;
									})() ? createComponent$1(Text, {
										class: "mt-1 text-sm text-muted whitespace-normal",
										get children() {
											return item.description;
										}
									}) : item.description;
								})];
							}
						})];
					}
				})
			});
		}
	});
}
//#endregion
export { Timeline };

//# sourceMappingURL=timeline.mjs.map