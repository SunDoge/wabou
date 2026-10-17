import { onCleanup } from "solid-js";
//#region src/primitives/animation-frame.ts
/**
* Drive explicit paint state from the native host's animation clock.
* Return `false` to stop scheduling frames before the owner is disposed.
*/
function createAnimationFrame(callback) {
	let frame = 0;
	let active = true;
	const tick = (timestamp) => {
		if (!active) return;
		if (callback(timestamp) === false) {
			active = false;
			return;
		}
		frame = requestAnimationFrame(tick);
	};
	frame = requestAnimationFrame(tick);
	const stop = () => {
		if (!active) return;
		active = false;
		cancelAnimationFrame(frame);
	};
	onCleanup(stop);
	return stop;
}
//#endregion
export { createAnimationFrame };

//# sourceMappingURL=animation-frame.mjs.map