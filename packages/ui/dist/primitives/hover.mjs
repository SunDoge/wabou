import { createSignal } from "solid-js";
//#region src/primitives/hover.ts
/** Reactive hover state and event bindings for a single target. */
function createHover() {
	const [hovered, setHovered] = createSignal(false);
	return {
		hovered,
		bindings: {
			onPointerEnter: () => setHovered(true),
			onPointerLeave: () => setHovered(false)
		}
	};
}
//#endregion
export { createHover };

//# sourceMappingURL=hover.mjs.map