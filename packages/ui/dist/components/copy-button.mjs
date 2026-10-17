import { Button } from "./button.mjs";
import { useClipboard } from "@wabou/core";
import { createSignal, omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/copy-button.tsx
function CopyButton(props) {
	const clipboard = useClipboard();
	const [copied, setCopied] = createSignal(false);
	const forwarded = omit(props, "value", "idleLabel", "copiedLabel", "copiedChildren", "onCopied", "onCopyError");
	const copy = async () => {
		try {
			await clipboard.writeText(props.value);
			setCopied(true);
			props.onCopied?.();
		} catch (error) {
			props.onCopyError?.(error);
		}
	};
	return createComponent$1(Button, mergeProps(forwarded, {
		get "aria-label"() {
			return props["aria-label"] ?? "Copy";
		},
		onClick: copy,
		get children() {
			return memo(() => {
				return !!copied();
			})() ? props.copiedChildren ?? props.copiedLabel ?? "Copied" : props.children ?? props.idleLabel ?? "Copy";
		}
	}));
}
//#endregion
export { CopyButton };

//# sourceMappingURL=copy-button.mjs.map