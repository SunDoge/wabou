import { createFocusWithin } from "../primitives/focus.mjs";
import { View } from "../primitives/view.mjs";
import { componentsDisabledControlClass } from "./theme.mjs";
import { Button } from "./button.mjs";
import { TextArea } from "./input.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, children, omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/prompt-composer.tsx
function promptComposerEditorHeightClass(value) {
	const lines = value.split("\n").length;
	if (lines >= 5 || value.length > 240) return "h-24";
	if (lines >= 3 || value.length > 120) return "h-20";
	if (lines >= 2 || value.length > 48) return "h-16";
	return "h-12";
}
/** Native multiline editor with density and chrome owned by PromptComposer. */
function PromptComposerEditor(props) {
	return createComponent$1(TextArea, mergeProps(props, {
		chrome: "none",
		get "class"() {
			return mergeClasses("min-w-0 px-0 py-1", promptComposerEditorHeightClass(props.value ?? ""), props.class);
		}
	}));
}
/** Stable circular primary action for a PromptComposer toolbar. */
function PromptComposerAction(props) {
	return createComponent$1(Button, mergeProps(props, {
		get variant() {
			return props.variant ?? "secondary";
		},
		get size() {
			return props.size ?? "icon";
		},
		get "class"() {
			return mergeClasses("flex-none rounded-full border border-subtle", props.class);
		}
	}));
}
function promptComposerClass(focused, invalid, disabled, className) {
	return mergeClasses("w-full min-w-0 rounded-xl border shadow-xs px-3 pt-2 pb-2 flex flex-col gap-2", invalid ? "border-danger" : focused ? "border-focus" : "border-subtle", className, componentsDisabledControlClass(disabled));
}
/** Shared compound surface for prompts, attachments, controls and status. */
function PromptComposer(props) {
	const focus = createFocusWithin();
	const forwarded = omit(props, "class", "children", "disabled", "invalid", "surfaceClass");
	return createComponent$1(View, mergeProps(forwarded, () => {
		return focus.bindings;
	}, {
		get role() {
			return props.role ?? "group";
		},
		get "aria-disabled"() {
			return props.disabled;
		},
		get "aria-invalid"() {
			return props.invalid;
		},
		"data-wabou-owns": "surface focus-ring",
		get "class"() {
			return mergeClasses(promptComposerClass(focus.focusWithin(), props.invalid ?? false, props.disabled ?? false, props.class), props.surfaceClass ?? "bg-input");
		},
		get children() {
			return props.children;
		}
	}));
}
/** Compact metadata row above the authored prompt. */
function PromptComposerStatus(props) {
	const content = children(() => props.children);
	const forwarded = omit(props, "children", "class");
	return createComponent$1(Show, {
		get when() {
			return content();
		},
		children: (resolved) => createComponent$1(View, mergeProps(forwarded, {
			get "class"() {
				return mergeClasses("w-full min-w-0 flex flex-row items-center justify-end gap-2", props.class);
			},
			get children() {
				return resolved();
			}
		}))
	});
}
/** Responsive action row below the authored prompt. */
function PromptComposerToolbar(props) {
	const forwarded = omit(props, "class", "wrap");
	return createComponent$1(View, mergeProps(forwarded, {
		get role() {
			return props.role ?? "toolbar";
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex flex-row flex-nowrap items-center justify-between gap-1.5", props.wrap && "flex-wrap", props.class);
		}
	}));
}
/** Shrink-safe group for the composer tools preceding its primary action. */
function PromptComposerTools(props) {
	const forwarded = omit(props, "class", "wrap");
	return createComponent$1(View, mergeProps(forwarded, {
		get role() {
			return props.role ?? "group";
		},
		get "class"() {
			return mergeClasses("min-w-0 flex-1 flex flex-row flex-nowrap items-center gap-0.5", props.wrap && "flex-wrap", props.class);
		}
	}));
}
//#endregion
export { PromptComposer, PromptComposerAction, PromptComposerEditor, PromptComposerStatus, PromptComposerToolbar, PromptComposerTools, promptComposerClass, promptComposerEditorHeightClass };

//# sourceMappingURL=prompt-composer.mjs.map