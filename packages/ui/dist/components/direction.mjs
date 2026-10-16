import { Text, View } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, createContext, useContext } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/direction.tsx
const DirectionContext = createContext("ltr");
/** Own logical direction in JavaScript instead of relying on web inheritance. */
function DirectionProvider(props) {
	return createComponent(DirectionContext, {
		get value() {
			return props.dir;
		},
		get children() {
			return props.children;
		}
	});
}
function useDirection() {
	return useContext(DirectionContext);
}
function DirectionalRow(props) {
	const direction = () => useDirection();
	return createComponent$1(View, mergeProps(props, { get "class"() {
		return mergeClasses("flex", direction() === "rtl" ? "flex-row-reverse" : "flex-row", props.class);
	} }));
}
function DirectionalText(props) {
	const direction = () => useDirection();
	return createComponent$1(Text, mergeProps(props, { get "class"() {
		return mergeClasses(direction() === "rtl" ? "text-right" : "text-left", props.class);
	} }));
}
//#endregion
export { DirectionProvider, DirectionalRow, DirectionalText, useDirection };

//# sourceMappingURL=direction.mjs.map