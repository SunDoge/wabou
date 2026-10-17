import { componentsDisabledControlClass, componentsDisabledItemClass } from "./theme.mjs";
import { mergeClasses } from "@wabou/core/style";
//#region src/components/select-semantics.ts
function pickerTriggerClass(variant, state) {
	if (state.disabled) return mergeClasses(variant === "default" ? "shadow-none" : "bg-transparent shadow-none", componentsDisabledControlClass(true));
	const focus = state.focused ? "border-focus" : "border-transparent";
	if (variant === "default") return mergeClasses("bg-input shadow-xs", state.focused ? "border-focus" : "border-subtle");
	return mergeClasses(state.pressed ? "bg-control-pressed" : state.hovered ? "bg-control-hover" : "bg-transparent", "shadow-none", focus);
}
/** Keep semantic ID references live for the same lifetime as the popup node. */
function selectControlsId(listboxId, open) {
	return open ? listboxId : void 0;
}
/** Shared visual contract for selectable and explicitly unavailable options. */
function pickerOptionClass(disabled, highlighted) {
	if (disabled) return componentsDisabledItemClass(true);
	if (highlighted) return "bg-control-hover text-primary cursor-pointer";
	return "bg-transparent text-secondary cursor-pointer";
}
//#endregion
export { pickerOptionClass, pickerTriggerClass, selectControlsId };

//# sourceMappingURL=select-semantics.mjs.map