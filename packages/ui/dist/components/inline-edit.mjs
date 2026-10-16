import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "./button.mjs";
import { Input } from "./input.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createSignal, untrack } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import pencil from "lucide-static/icons/pencil.svg?raw";
//#region src/components/inline-edit.tsx
/** Compact rename interaction with explicit Enter, Escape, and blur behavior. */
function InlineEdit(props) {
	const [internal, setInternal] = createSignal(props.defaultValue ?? "");
	const value = () => props.value ?? internal();
	const [editing, setEditing] = createSignal(false);
	const [draft, setDraft] = createSignal(untrack(value));
	const begin = () => {
		if (props.disabled) return;
		setDraft(untrack(value));
		setEditing(true);
	};
	const commit = () => {
		if (!untrack(editing)) return;
		const next = untrack(draft);
		const previous = untrack(value);
		setEditing(false);
		if (props.value === void 0) setInternal(next);
		if (next !== previous) props.onValueChange?.(next);
		props.onCommit?.(next);
	};
	const cancel = () => {
		if (!untrack(editing)) return;
		setDraft(untrack(value));
		setEditing(false);
		props.onCancel?.();
	};
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("min-w-0", props.class);
		},
		get children() {
			return memo(() => {
				return !!editing();
			})() ? createComponent$1(Input, {
				ref: (node) => {
					node.focus();
				},
				get "aria-label"() {
					return props["aria-label"];
				},
				get "class"() {
					return mergeClasses("min-w-0", props.inputClass);
				},
				get value() {
					return draft();
				},
				get placeholder() {
					return props.placeholder;
				},
				onInput: (event) => setDraft(event.currentTarget.value),
				onBlur: commit,
				onKeyDown: (event) => {
					if (event.key === "Enter") {
						event.preventDefault();
						commit();
					} else if (event.key === "Escape") {
						event.preventDefault();
						cancel();
					}
				}
			}) : createComponent$1(Button, {
				variant: "ghost",
				get disabled() {
					return props.disabled;
				},
				get "aria-label"() {
					return `Edit ${props["aria-label"]}`;
				},
				get "class"() {
					return mergeClasses("h-8 max-w-full min-w-0 px-2 justify-start gap-2", props.displayClass);
				},
				onClick: begin,
				get children() {
					return [createComponent$1(Text, {
						class: "min-w-0 flex-1 text-sm text-primary",
						maxLines: 1,
						get children() {
							return value() || props.placeholder || "Untitled";
						}
					}), createComponent$1(Icon, {
						"aria-hidden": "true",
						class: "flex-none text-muted",
						source: pencil,
						size: 14
					})];
				}
			});
		}
	});
}
//#endregion
export { InlineEdit };

//# sourceMappingURL=inline-edit.mjs.map