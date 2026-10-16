import { PasswordInput as PasswordInput$1, TextArea as TextArea$1, TextInput } from "../primitives/view.mjs";
import { componentsControlContentSize, componentsControlSize, componentsDisabledContentClass, componentsDisabledControlClass } from "./theme.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/input.tsx
/** A plain-text input. Secrets must use `PasswordInput`. */
function Input(props) {
	const forwarded = omit(props, "chrome", "surfaceClass");
	return createComponent$1(TextInput, mergeProps(forwarded, {
		get "data-wabou-owns"() {
			return (props.chrome ?? "default") === "default" ? "surface native-editor" : "native-editor";
		},
		get "class"() {
			return mergeClasses("w-full flex items-center py-1 text-primary", (props.chrome ?? "default") === "default" ? componentsControlSize("default") : componentsControlContentSize("default"), (props.chrome ?? "default") === "default" && mergeClasses("border border-subtle shadow-xs", props.surfaceClass ?? "bg-input"), props.class, (props.chrome ?? "default") === "default" ? componentsDisabledControlClass(props.disabled ?? false) : componentsDisabledContentClass(props.disabled ?? false));
		}
	}));
}
/** A native secret input whose value never crosses into JavaScript. */
function PasswordInput(props) {
	return createComponent$1(PasswordInput$1, mergeProps(props, { get "class"() {
		return mergeClasses("w-full flex items-center py-1 border shadow-xs", componentsControlSize("default"), "border-subtle bg-input text-primary", props.class, componentsDisabledControlClass(props.disabled ?? false));
	} }));
}
function TextArea(props) {
	const forwarded = omit(props, "chrome", "surfaceClass");
	return createComponent$1(TextArea$1, mergeProps(forwarded, {
		get "data-wabou-owns"() {
			return (props.chrome ?? "default") === "default" ? "surface native-editor" : "native-editor";
		},
		get "class"() {
			return mergeClasses("h-24 w-full px-2.5 py-2 text-sm leading-normal text-primary", (props.chrome ?? "default") === "default" && mergeClasses("rounded-md border border-subtle shadow-xs", props.surfaceClass ?? "bg-input"), props.class, (props.chrome ?? "default") === "default" ? componentsDisabledControlClass(props.disabled ?? false) : componentsDisabledContentClass(props.disabled ?? false));
		}
	}));
}
//#endregion
export { Input, PasswordInput, TextArea };

//# sourceMappingURL=input.mjs.map