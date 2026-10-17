import { View } from "./view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, mergeProps } from "@wabou/core/renderer";
//#region src/primitives/layout.tsx
/** Horizontal flex container. No wrapper node is added beyond the host View. */
function Row(props) {
	return createComponent(View, mergeProps(props, { get "class"() {
		return mergeClasses("flex flex-row", props.class);
	} }));
}
/** Vertical flex container. No wrapper node is added beyond the host View. */
function Column(props) {
	return createComponent(View, mergeProps(props, { get "class"() {
		return mergeClasses("flex flex-col", props.class);
	} }));
}
/** Flex container that centers children on both axes. */
function Center(props) {
	return createComponent(View, mergeProps(props, { get "class"() {
		return mergeClasses("flex items-center justify-center", props.class);
	} }));
}
//#endregion
export { Center, Column, Row };

//# sourceMappingURL=layout.mjs.map