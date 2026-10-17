import { View } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/aspect-ratio.tsx
function aspectRatioStyle(ratio, style) {
	const resolved = ratio ?? 1;
	if (!Number.isFinite(resolved) || resolved <= 0) throw new RangeError("AspectRatio ratio must be a finite positive number");
	return {
		...style,
		"aspect-ratio": resolved
	};
}
/** A native Taffy aspect-ratio constraint with explicit overflow ownership. */
function AspectRatio(props) {
	const rest = omit(props, "ratio", "style", "class", "children");
	return createComponent$1(View, mergeProps(rest, {
		"data-wabou-owns": "clip",
		get "class"() {
			return mergeClasses("w-full min-w-0 overflow-hidden", props.class);
		},
		get style() {
			return aspectRatioStyle(props.ratio, props.style);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { AspectRatio, aspectRatioStyle };

//# sourceMappingURL=aspect-ratio.mjs.map