import { Icon, Text, View } from "../primitives/view.mjs";
import { CopyButton } from "./copy-button.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, memo } from "@wabou/core/renderer";
import check from "lucide-static/icons/check.svg?raw";
import copy from "lucide-static/icons/copy.svg?raw";
//#region src/components/code-block.tsx
function CodeBlock(props) {
	return createComponent(View, {
		role: "group",
		get "aria-label"() {
			return props["aria-label"] ?? "Code block";
		},
		get "class"() {
			return mergeClasses("min-w-0 overflow-hidden rounded-xl border border-subtle bg-control", props.class);
		},
		get children() {
			return [createComponent(View, {
				class: "h-8 flex flex-row items-center justify-between gap-3 pl-3 pr-1 bg-control",
				get children() {
					return [createComponent(Text, {
						class: "min-w-0 font-mono text-xs text-secondary",
						get children() {
							return props.language ?? "text";
						}
					}), memo(() => {
						return memo(() => {
							return props.copyable !== false;
						})() && createComponent(CopyButton, {
							get value() {
								return props.code;
							},
							variant: "ghost",
							size: "icon",
							class: "w-7 h-7 text-muted",
							get idleLabel() {
								return props.copyLabel;
							},
							get copiedLabel() {
								return props.copiedLabel;
							},
							get "aria-label"() {
								return props.copyLabel;
							},
							get copiedChildren() {
								return createComponent(Icon, {
									source: check,
									size: 13
								});
							},
							get children() {
								return createComponent(Icon, {
									source: copy,
									size: 13
								});
							}
						});
					})];
				}
			}), createComponent(Text, {
				class: "min-w-0 p-3 font-mono text-sm text-primary whitespace-normal",
				get children() {
					return props.code;
				}
			})];
		}
	});
}
//#endregion
export { CodeBlock };

//# sourceMappingURL=code-block.mjs.map