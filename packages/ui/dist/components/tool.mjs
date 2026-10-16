import { Icon, Text, View } from "../primitives/view.mjs";
import { Badge } from "./badge.mjs";
import { CodeBlock } from "./code-block.mjs";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "./disclosure.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
import checkCircle from "lucide-static/icons/circle-check.svg?raw";
import circleX from "lucide-static/icons/circle-x.svg?raw";
import ellipsisCircle from "lucide-static/icons/circle-ellipsis.svg?raw";
import clock from "lucide-static/icons/clock-3.svg?raw";
import wrench from "lucide-static/icons/wrench.svg?raw";
//#region src/components/tool.tsx
/** Composable disclosure root for an AI or automation tool invocation. */
function Tool(props) {
	const rest = omit(props, "class");
	return createComponent$1(Collapsible, mergeProps(rest, { get "class"() {
		return mergeClasses("w-full min-w-0 overflow-hidden rounded-lg border border-subtle bg-surface shadow-xs", props.class);
	} }));
}
function statusPresentation(status) {
	return match(status).with("pending", () => ({
		icon: ellipsisCircle,
		label: "Pending",
		variant: "secondary"
	})).with("running", () => ({
		icon: clock,
		label: "Running",
		variant: "secondary"
	})).with("success", () => ({
		icon: checkCircle,
		label: "Completed",
		variant: "success"
	})).with("failed", () => ({
		icon: circleX,
		label: "Failed",
		variant: "destructive"
	})).exhaustive();
}
function toolHeaderLabel(title, summary, status) {
	return [
		title,
		summary,
		status ? statusPresentation(status).label : void 0
	].filter(Boolean).join(": ");
}
/** Stable title, summary, status and disclosure geometry for one tool call. */
function ToolHeader(props) {
	const presentation = () => props.status ? statusPresentation(props.status) : void 0;
	const rest = omit(props, "title", "summary", "status", "icon", "class", "aria-label");
	return createComponent$1(CollapsibleTrigger, mergeProps(rest, {
		get "aria-label"() {
			return props["aria-label"] ?? toolHeaderLabel(props.title, props.summary, props.status);
		},
		get "class"() {
			return mergeClasses("min-h-10 px-3 py-1.5 bg-control text-left", props.class);
		},
		get children() {
			return createComponent$1(View, {
				class: "min-w-0 flex-1 flex flex-row items-center gap-2",
				get children() {
					return [
						createComponent$1(Icon, {
							get source() {
								return props.icon ?? wrench;
							},
							size: 14,
							class: "flex-none text-muted"
						}),
						createComponent$1(Text, {
							class: "min-w-0 max-w-2/5 truncate text-sm font-semibold text-primary",
							get children() {
								return props.title;
							}
						}),
						memo(() => {
							return memo(() => {
								return !!props.summary;
							})() ? createComponent$1(Text, {
								class: "min-w-0 flex-1 truncate text-xs text-muted",
								get children() {
									return props.summary;
								}
							}) : props.summary;
						}),
						memo(() => {
							return memo(() => {
								return !!presentation();
							})() ? createComponent$1(Badge, {
								get variant() {
									return presentation()?.variant;
								},
								weight: "normal",
								class: "flex flex-row items-center gap-1",
								get children() {
									return [createComponent$1(Icon, {
										get source() {
											return presentation()?.icon ?? wrench;
										},
										size: 11
									}), memo(() => {
										return presentation()?.label;
									})];
								}
							}) : presentation();
						})
					];
				}
			});
		}
	}));
}
function ToolContent(props) {
	const rest = omit(props, "class");
	return createComponent$1(CollapsibleContent, mergeProps(rest, { get "class"() {
		return mergeClasses("min-w-0 border-t border-subtle bg-surface", props.class);
	} }));
}
/** Labelled code payload used for tool parameters, results and errors. */
function ToolCodeSection(props) {
	const rest = omit(props, "code", "label", "language", "copyable", "class", "codeClass", "codeProps");
	return createComponent$1(View, mergeProps(rest, {
		get "class"() {
			return mergeClasses("min-w-0 p-3 gap-2", props.class);
		},
		get children() {
			return [createComponent$1(Text, {
				class: "text-xs font-medium tracking-wide text-muted",
				get children() {
					return props.label;
				}
			}), createComponent$1(CodeBlock, mergeProps(() => {
				return props.codeProps;
			}, {
				get code() {
					return props.code;
				},
				get language() {
					return props.language;
				},
				get copyable() {
					return props.copyable;
				},
				get "class"() {
					return mergeClasses("rounded-md", props.codeClass);
				}
			}))];
		}
	}));
}
function ToolInput(props) {
	return createComponent$1(ToolCodeSection, mergeProps(props, { get label() {
		return props.label ?? "Parameters";
	} }));
}
function ToolOutput(props) {
	const rest = omit(props, "error");
	return createComponent$1(ToolCodeSection, mergeProps(rest, {
		get label() {
			return props.label ?? (props.error ? "Error" : "Result");
		},
		get "class"() {
			return mergeClasses("border-t border-subtle", props.error ? "bg-danger-surface" : void 0, props.class);
		}
	}));
}
//#endregion
export { Tool, ToolCodeSection, ToolContent, ToolHeader, ToolInput, ToolOutput, toolHeaderLabel };

//# sourceMappingURL=tool.mjs.map