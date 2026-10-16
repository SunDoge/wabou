import { createMemo } from "solid-js";
//#region src/animation/native.ts
const identity = [
	1,
	0,
	0,
	1,
	0,
	0
];
function read(value) {
	return typeof value === "function" ? value() : value;
}
/**
* Publish spring targets to the native renderer.
*
* The accessor changes only when its authored target changes. Rust retains the
* sampled value and velocity between publications and across ordinary JS HMR.
*/
function useSpring(target, options = {}) {
	let first = true;
	return createMemo(() => {
		const next = read(target);
		const reducedMotion = options.reducedMotion ? read(options.reducedMotion) : false;
		const initial = first && options.initial ? options.initial : void 0;
		const snapInitially = first && !initial;
		first = false;
		const initialTransform = snapInitially ? next.transform ?? identity : initial?.transform;
		const initialOpacity = snapInitially ? next.opacity : initial?.opacity;
		return {
			kind: "spring",
			response: options.response ?? .5,
			damping: options.damping ?? .82,
			epsilon: options.restDelta ?? .001,
			targetTransform: next.transform ?? identity,
			reducedMotion,
			...options.mass === void 0 ? {} : { mass: options.mass },
			...options.stiffness === void 0 ? {} : { stiffness: options.stiffness },
			...next.opacity === void 0 ? {} : { targetOpacity: next.opacity },
			...initialTransform === void 0 ? {} : { initialTransform },
			...initialOpacity === void 0 ? {} : { initialOpacity }
		};
	});
}
/** Publish an interruptible finite tween to the native renderer. */
function useTween(target, options = {}) {
	let generation = 0;
	let first = true;
	return createMemo(() => {
		const next = read(target);
		const initial = first && options.initial ? options.initial : void 0;
		const snapInitially = first && !initial;
		first = false;
		generation += 1;
		const reducedMotion = options.reducedMotion ? read(options.reducedMotion) : false;
		const fromTransform = snapInitially ? next.transform ?? identity : initial?.transform;
		const fromOpacity = snapInitially ? next.opacity : initial?.opacity;
		return {
			kind: "transition",
			generation,
			duration: reducedMotion || snapInitially ? 0 : options.duration ?? .2,
			easing: options.easing ?? "easeOut",
			...fromTransform === void 0 ? {} : { fromTransform },
			...next.transform === void 0 ? {} : { toTransform: next.transform },
			...fromOpacity === void 0 ? {} : { fromOpacity },
			...next.opacity === void 0 ? {} : { toOpacity: next.opacity }
		};
	});
}
//#endregion
export { useSpring, useTween };

//# sourceMappingURL=native.mjs.map