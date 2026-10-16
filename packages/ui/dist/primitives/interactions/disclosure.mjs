import { unchanged } from "./machine.mjs";
import { createControllableState } from "./state.mjs";
import { match } from "ts-pattern";
//#region src/primitives/interactions/disclosure.ts
function updateDisclosure(state, event) {
	return match(event).with({ type: "DISABLED" }, ({ disabled }) => ({
		state: {
			...state,
			disabled,
			open: disabled ? false : state.open
		},
		commands: []
	})).with({ type: "OPEN" }, () => state.disabled || state.open ? unchanged(state) : {
		state: {
			...state,
			open: true
		},
		commands: []
	}).with({ type: "CLOSE" }, () => !state.open ? unchanged(state) : {
		state: {
			...state,
			open: false
		},
		commands: []
	}).with({ type: "TOGGLE" }, () => state.disabled ? unchanged(state) : {
		state: {
			...state,
			open: !state.open
		},
		commands: []
	}).exhaustive();
}
function createDisclosure(options = {}) {
	const controlled = createControllableState({
		value: options.open ?? (() => void 0),
		defaultValue: options.defaultOpen ?? false,
		disabled: options.disabled,
		onChange: options.onOpenChange
	});
	const set = (type) => {
		const result = updateDisclosure({
			open: controlled.value(),
			disabled: options.disabled?.() ?? false
		}, { type });
		return controlled.set(result.state.open);
	};
	return {
		open: controlled.value,
		disabled: () => options.disabled?.() ?? false,
		openDisclosure: () => set("OPEN"),
		close: () => set("CLOSE"),
		toggle: () => set("TOGGLE")
	};
}
//#endregion
export { createDisclosure, updateDisclosure };

//# sourceMappingURL=disclosure.mjs.map