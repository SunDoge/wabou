import { createSignal } from "solid-js";
//#region src/primitives/press.ts
/** Reactive pointer-press state with disabled-aware activation. */
function createPress(options = {}) {
	const [pressed, setPressed] = createSignal(false);
	const disabled = () => typeof options.disabled === "function" ? options.disabled() : !!options.disabled;
	const release = () => setPressed(false);
	return {
		pressed,
		bindings: {
			onPointerDown: () => {
				if (!disabled()) setPressed(true);
			},
			onPointerUp: release,
			onPointerCancel: release,
			onPointerLeave: release,
			onClick: (event) => {
				if (!disabled()) options.onPress?.(event);
			}
		}
	};
}
/** CSS `:active`-like state without an activation callback. */
function createActive(disabled) {
	const { pressed, bindings } = createPress({ disabled });
	return {
		active: pressed,
		bindings
	};
}
//#endregion
export { createActive, createPress };

//# sourceMappingURL=press.mjs.map