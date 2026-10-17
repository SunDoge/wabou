import { Text, View } from "../primitives/view.mjs";
import { Card, CardContent } from "./card.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, memo } from "@wabou/core/renderer";
//#region src/components/stat-card.tsx
function StatCard(props) {
	return createComponent(Card, {
		role: "group",
		get "aria-label"() {
			return props.label;
		},
		get "class"() {
			return props.class;
		},
		get children() {
			return createComponent(CardContent, {
				class: "gap-2",
				get children() {
					return [
						createComponent(View, {
							class: "flex flex-row items-center justify-between gap-3",
							get children() {
								return [createComponent(Text, {
									class: "min-w-0 text-xs font-medium text-muted",
									get children() {
										return props.label;
									}
								}), createComponent(View, {
									"aria-hidden": "true",
									get "class"() {
										return mergeClasses("w-2 h-2 rounded-full bg-accent", props.indicatorClass);
									}
								})];
							}
						}),
						createComponent(Text, {
							class: "text-3xl font-semibold text-primary",
							get children() {
								return props.value;
							}
						}),
						memo(() => {
							return memo(() => {
								return !!(props.description || props.trend);
							})() ? createComponent(View, {
								class: "min-w-0 flex flex-row items-center gap-2",
								get children() {
									return [memo(() => {
										return memo(() => {
											return !!props.trend;
										})() ? createComponent(Text, {
											class: "flex-none text-xs font-medium text-success-primary",
											get children() {
												return props.trend;
											}
										}) : props.trend;
									}), memo(() => {
										return memo(() => {
											return !!props.description;
										})() ? createComponent(Text, {
											class: "min-w-0 text-xs text-muted whitespace-normal",
											get children() {
												return props.description;
											}
										}) : props.description;
									})];
								}
							}) : props.description || props.trend;
						})
					];
				}
			});
		}
	});
}
//#endregion
export { StatCard };

//# sourceMappingURL=stat-card.mjs.map