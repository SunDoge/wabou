import { createEffect } from "solid-js";
//#region src/primitives/scroll-reset.ts
/** Reset one explicitly selected native viewport after its key changes. */
function createScrollReset(options) {
	const reset = () => {
		options.target()?.scrollTo({
			left: options.left ?? 0,
			top: options.top ?? 0
		});
	};
	createEffect(options.key, reset, { defer: true });
	return reset;
}
//#endregion
export { createScrollReset };

//# sourceMappingURL=scroll-reset.mjs.map