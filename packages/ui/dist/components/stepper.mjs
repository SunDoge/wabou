import { Text, View } from "../primitives/view.mjs";
import { Button } from "./button.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, createSignal } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
//#region src/components/stepper.tsx
function Stepper(props) {
	const [local, setLocal] = createSignal(props.defaultValue ?? props.steps[0]?.id ?? "");
	const value = () => props.value ?? local();
	const currentIndex = () => Math.max(0, props.steps.findIndex((step) => step.id === value()));
	const select = (id) => {
		if (props.value === void 0) setLocal(id);
		props.onValueChange?.(id);
	};
	return createComponent$1(View, {
		role: "group",
		get "aria-label"() {
			return props["aria-label"] ?? "Progress steps";
		},
		get "class"() {
			return mergeClasses("min-w-0 flex flex-row items-start", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return props.steps;
				},
				children: (step, index) => createComponent$1(View, {
					class: "min-w-0 flex-1 flex flex-row items-start",
					get children() {
						return [createComponent$1(View, {
							class: "min-w-0 flex-1 items-center gap-2",
							get children() {
								return [
									createComponent$1(Button, {
										get "aria-label"() {
											return `Go to ${step.label}`;
										},
										get variant() {
											return index() <= currentIndex() ? "default" : "outline";
										},
										size: "icon",
										onClick: () => select(step.id),
										get children() {
											return String(index() + 1);
										}
									}),
									createComponent$1(Text, {
										class: "text-sm font-medium text-primary",
										get children() {
											return step.label;
										}
									}),
									memo(() => {
										return memo(() => {
											return !!step.description;
										})() ? createComponent$1(Text, {
											class: "text-xs text-muted whitespace-normal",
											get children() {
												return step.description;
											}
										}) : step.description;
									})
								];
							}
						}), memo(() => {
							return memo(() => {
								return index() < props.steps.length - 1;
							})() && createComponent$1(View, { get "class"() {
								return mergeClasses("h-px flex-1 mt-4", index() < currentIndex() ? "bg-accent" : "bg-subtle");
							} });
						})];
					}
				})
			});
		}
	});
}
//#endregion
export { Stepper };

//# sourceMappingURL=stepper.mjs.map