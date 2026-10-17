import { Text } from "../primitives/view.mjs";
import { componentToneClass } from "./tone.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/badge.tsx
function badgeClass(variant = "default", weight = "medium", className, size = "default") {
	const colors = match(variant).with("default", () => "bg-accent border-accent text-on-accent").with("secondary", () => "bg-control border-subtle text-primary").with("outline", () => "bg-transparent border-strong text-secondary").with("ghost", () => "bg-transparent border-transparent text-secondary").with("link", () => "bg-transparent border-transparent text-accent").with("info", () => componentToneClass("accent", "surface")).with("success", () => componentToneClass("success", "surface")).with("destructive", () => componentToneClass("danger", "surface")).exhaustive();
	return mergeClasses("flex-none overflow-hidden whitespace-nowrap flex items-center justify-center rounded-full border", match(size).with("sm", () => "h-4 px-1.5 text-xs").with("default", () => "h-5 px-2 py-0.5 text-xs").with("lg", () => "h-6 px-2.5 py-0.5 text-sm").exhaustive(), weight === "normal" ? "font-normal" : "font-medium", colors, className);
}
/** Compact status text with shadcn-compatible visual variants. */
function Badge(props) {
	const rest = omit(props, "variant", "size", "weight", "class", "children");
	return createComponent$1(Text, mergeProps(rest, {
		get "class"() {
			return badgeClass(props.variant, props.weight, props.class, props.size);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { Badge, badgeClass };

//# sourceMappingURL=badge.mjs.map