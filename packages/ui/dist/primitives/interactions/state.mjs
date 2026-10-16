import { createSignal } from "solid-js";
//#region src/primitives/interactions/state.ts
function createControllableState(options) {
	const [local, setLocal] = createSignal({ value: options.defaultValue }, { ownedWrite: true });
	const value = () => {
		const controlled = options.value();
		return controlled === void 0 ? local().value : controlled;
	};
	return {
		value,
		set(next) {
			if (options.disabled?.() || Object.is(value(), next)) return false;
			if (options.value() === void 0) setLocal({ value: next });
			options.onChange?.(next);
			return true;
		}
	};
}
//#endregion
export { createControllableState };

//# sourceMappingURL=state.mjs.map