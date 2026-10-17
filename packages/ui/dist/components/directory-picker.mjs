import { Icon } from "../primitives/view.mjs";
import { directoryPickerOptions } from "./directory-picker-state.mjs";
import { InputGroup, InputGroupButton, InputGroupInput } from "./forms.mjs";
import { createAsyncAction, useDialog } from "@wabou/core";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import folder from "lucide-static/icons/folder.svg?raw";
//#region src/components/directory-picker.tsx
/** A controlled path input paired with the operating system directory picker. */
function DirectoryPicker(props) {
	const nativeDialog = useDialog();
	const local = props;
	const selection = createAsyncAction(() => nativeDialog.pickDirectory(directoryPickerOptions(local.value, local.dialogOptions)));
	const inputProps = omit(props, "value", "onValueChange", "dialogOptions", "browseLabel", "pendingLabel", "browseAriaLabel", "class", "inputClass", "buttonClass", "onBrowseError", "onBrowseSelect");
	async function browse() {
		if (inputProps.disabled) return;
		const result = await selection.run();
		if (!result.ok) {
			if (local.onBrowseError) local.onBrowseError(result.error);
			else throw result.error;
			return;
		}
		if (result.value !== null) {
			local.onValueChange(result.value);
			local.onBrowseSelect?.(result.value);
		}
	}
	return createComponent$1(InputGroup, {
		get disabled() {
			return Boolean(inputProps.disabled) || selection.pending();
		},
		get "class"() {
			return local.class;
		},
		get children() {
			return [createComponent$1(InputGroupInput, mergeProps(inputProps, {
				get "class"() {
					return local.inputClass;
				},
				get value() {
					return local.value;
				},
				onInput: (event) => local.onValueChange(event.currentTarget.value)
			})), createComponent$1(InputGroupButton, {
				get "class"() {
					return mergeClasses("flex-none", local.buttonClass);
				},
				get disabled() {
					return Boolean(inputProps.disabled) || selection.pending();
				},
				get "aria-label"() {
					return local.browseAriaLabel ?? local.browseLabel ?? "Browse directory";
				},
				onClick: () => void browse(),
				get children() {
					return [createComponent$1(Icon, {
						source: folder,
						size: 14
					}), memo(() => {
						return memo(() => {
							return !!selection.pending();
						})() ? local.pendingLabel ?? "Opening…" : local.browseLabel ?? "Browse…";
					})];
				}
			})];
		}
	});
}
//#endregion
export { DirectoryPicker };

//# sourceMappingURL=directory-picker.mjs.map