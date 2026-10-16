import { Icon, Text } from "../primitives/view.mjs";
import { Pulse } from "../primitives/motion.mjs";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "./disclosure.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import sparkles from "lucide-static/icons/sparkles.svg?raw";
//#region src/components/reasoning.tsx
/** Quiet disclosure surface for model reasoning or intermediate thought. */
function Reasoning(props) {
	const rest = omit(props, "class");
	return createComponent$1(Collapsible, mergeProps(rest, { get "class"() {
		return mergeClasses("w-full min-w-0 overflow-hidden rounded-lg border border-subtle bg-surface-muted", props.class);
	} }));
}
function ReasoningTrigger(props) {
	const label = () => props.label ?? (props.streaming ? "Thinking" : "Reasoning");
	const rest = omit(props, "label", "streaming", "class", "aria-label");
	return createComponent$1(CollapsibleTrigger, mergeProps(rest, {
		get "aria-label"() {
			return props["aria-label"] ?? label();
		},
		get "class"() {
			return mergeClasses("min-h-9 px-3 py-1.5 text-left", props.class);
		},
		get children() {
			return [
				createComponent$1(Icon, {
					source: sparkles,
					size: 13,
					class: "flex-none text-muted"
				}),
				createComponent$1(Text, {
					class: "min-w-0 flex-1 text-sm font-medium text-secondary",
					get children() {
						return label();
					}
				}),
				memo(() => {
					return memo(() => {
						return !!props.streaming;
					})() ? createComponent$1(Pulse, {
						"aria-hidden": "true",
						class: "w-1.5 h-1.5 flex-none rounded-full bg-accent",
						from: .3,
						to: 1,
						duration: .8
					}) : props.streaming;
				})
			];
		}
	}));
}
function ReasoningContent(props) {
	const rest = omit(props, "class");
	return createComponent$1(CollapsibleContent, mergeProps(rest, { get "class"() {
		return mergeClasses("min-w-0 border-t border-subtle px-3 py-2", props.class);
	} }));
}
//#endregion
export { Reasoning, ReasoningContent, ReasoningTrigger };

//# sourceMappingURL=reasoning.mjs.map