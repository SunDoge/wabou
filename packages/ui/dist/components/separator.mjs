import { View } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/separator.tsx
/** A visual divider with an opt-in semantic separator contract. */
function Separator(props) {
	const orientation = () => props.orientation ?? "horizontal";
	const decorative = () => props.decorative ?? true;
	const dimensions = () => match(orientation()).with("horizontal", () => "h-px w-full").with("vertical", () => "w-px h-full").exhaustive();
	const rest = omit(props, "class", "decorative", "orientation");
	return createComponent$1(View, mergeProps(rest, {
		get role() {
			return decorative() ? "presentation" : "separator";
		},
		get "aria-hidden"() {
			return decorative() ? "true" : void 0;
		},
		get "aria-orientation"() {
			return memo(() => {
				return !!decorative();
			})() ? void 0 : orientation();
		},
		get "class"() {
			return mergeClasses("flex-none bg-subtle", dimensions(), props.class);
		}
	}));
}
/**
* A horizontal divider whose compact center content names the boundary.
*
* Keep the content short. Interactive content is supported so disclosures can
* explain what happened between two regions without turning into a full row.
*/
function LabeledSeparator(props) {
	const rest = omit(props, "class", "children");
	return createComponent$1(View, mergeProps(rest, {
		get "class"() {
			return mergeClasses("w-full min-w-0 h-7 flex flex-row items-center gap-3", props.class);
		},
		get children() {
			return [
				createComponent$1(Separator, { class: "min-w-0 flex-1" }),
				createComponent$1(View, {
					class: "min-w-0 max-w-4/5 flex-none flex items-center justify-center",
					get children() {
						return props.children;
					}
				}),
				createComponent$1(Separator, { class: "min-w-0 flex-1" })
			];
		}
	}));
}
//#endregion
export { LabeledSeparator, Separator };

//# sourceMappingURL=separator.mjs.map