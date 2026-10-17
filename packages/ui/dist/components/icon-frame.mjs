import { Icon } from "../primitives/view.mjs";
import { Center } from "../primitives/layout.mjs";
import { componentToneClass } from "./tone.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/icon-frame.tsx
const frameSizeClass = (size) => match(size).with("sm", () => "w-8 h-8 rounded-md").with("default", () => "w-10 h-10 rounded-lg").with("lg", () => "w-12 h-12 rounded-lg").with("xl", () => "w-14 h-14 rounded-2xl").exhaustive();
const defaultIconSize = (size) => match(size).with("sm", () => 16).with("default", () => 20).with("lg", () => 23).with("xl", () => 26).exhaustive();
const variantClass = (variant, tone) => match(variant).with("plain", () => mergeClasses("border-transparent bg-transparent", componentToneClass(tone, "text"))).with("soft", () => componentToneClass(tone, "surface")).with("solid", () => componentToneClass(tone, "solid")).exhaustive();
/**
* A square icon surface whose geometry is correct by construction.
*
* Use this for standalone icon tiles. Buttons and menu items already own their
* icon alignment and should continue to use `Icon` directly.
*/
function IconFrame(props) {
	const rest = omit(props, "source", "fill", "label", "size", "iconSize", "tone", "variant", "iconClass", "class");
	const size = () => props.size ?? "default";
	return createComponent$1(Center, mergeProps(rest, {
		role: "presentation",
		get "class"() {
			return mergeClasses("flex-none", frameSizeClass(size()), variantClass(props.variant ?? "plain", props.tone ?? "neutral"), props.class);
		},
		get children() {
			return createComponent$1(Icon, {
				get source() {
					return props.source;
				},
				get size() {
					return props.iconSize ?? defaultIconSize(size());
				},
				get fill() {
					return props.fill;
				},
				get label() {
					return props.label;
				},
				get "class"() {
					return props.iconClass;
				}
			});
		}
	}));
}
//#endregion
export { IconFrame };

//# sourceMappingURL=icon-frame.mjs.map