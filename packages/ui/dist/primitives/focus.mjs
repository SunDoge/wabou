import { createSignal } from "solid-js";
//#region src/primitives/focus.ts
let keyboardModality = false;
/** Reactive focus state and event bindings for a single target. */
function createFocus() {
	const [focused, setFocused] = createSignal(false);
	const [focusVisible, setFocusVisible] = createSignal(false);
	return {
		focused,
		focusVisible,
		pointerModality: () => {
			keyboardModality = false;
			if (focused()) setFocusVisible(false);
		},
		keyboardModality: () => {
			keyboardModality = true;
			if (focused()) setFocusVisible(true);
		},
		bindings: {
			onFocus: (event) => {
				setFocused(true);
				setFocusVisible(event?.payload?.focusVisible ?? keyboardModality);
			},
			onBlur: () => {
				setFocused(false);
				setFocusVisible(false);
			}
		}
	};
}
/** Reactive equivalent of `:focus-within`, using bubbling focus events. */
function createFocusWithin() {
	const [focusWithin, setFocusWithin] = createSignal(false);
	return {
		focusWithin,
		bindings: {
			onFocusIn: () => setFocusWithin(true),
			onFocusOut: () => setFocusWithin(false)
		}
	};
}
//#endregion
export { createFocus, createFocusWithin };

//# sourceMappingURL=focus.mjs.map