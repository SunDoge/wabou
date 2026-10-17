import { Text } from "../primitives/view.mjs";
import { componentsDisabledInteractiveClass } from "./theme.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/label.tsx
function resolveControl(control) {
	return typeof control === "function" ? control() : control;
}
/** Text label that forwards pointer activation to an explicit native control. */
function Label(props) {
	const rest = omit(props, "class", "children", "disabled", "control", "onClick");
	return createComponent$1(Text, mergeProps(rest, {
		role: "label",
		get "aria-disabled"() {
			return props.disabled;
		},
		get "class"() {
			return mergeClasses("min-w-0 text-sm font-medium text-primary", props.disabled ? "" : "cursor-pointer", props.class, componentsDisabledInteractiveClass(props.disabled ?? false));
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!props.disabled && !event.defaultPrevented) resolveControl(props.control)?.focus();
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { Label };

//# sourceMappingURL=label.mjs.map