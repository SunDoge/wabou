import { useWindow } from "@wabou/core";
import { rotate2d, translate2d } from "@wabou/core/style";
import { createContext, createEffect, createMemo, createSignal, onCleanup, untrack, useContext } from "solid-js";
import { createComponent as createComponent$1 } from "@wabou/core/renderer";
//#region src/animation/config.tsx
const DEFAULT_MOTION_CONFIG = Object.freeze({ reducedMotion: () => useWindow().reducedMotion() });
const MotionConfigContext = createContext(DEFAULT_MOTION_CONFIG);
/** Application-level motion policy inherited by all styled Wabou components. */
function MotionConfigProvider(props) {
	const parent = useMotionConfig();
	return createComponent$1(MotionConfigContext, {
		value: { reducedMotion: () => props.reducedMotion ?? parent.reducedMotion() },
		get children() {
			return props.children;
		}
	});
}
function useMotionConfig() {
	return useContext(MotionConfigContext);
}
function useReducedMotion() {
	return useMotionConfig().reducedMotion;
}
//#endregion
//#region src/animation/native.ts
const identity = [
	1,
	0,
	0,
	1,
	0,
	0
];
function read$1(value) {
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
		const next = read$1(target);
		const reducedMotion = options.reducedMotion ? read$1(options.reducedMotion) : false;
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
		const next = read$1(target);
		const initial = first && options.initial ? options.initial : void 0;
		const snapInitially = first && !initial;
		first = false;
		generation += 1;
		const reducedMotion = options.reducedMotion ? read$1(options.reducedMotion) : false;
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
//#region src/animation/index.ts
const hasNativeMotion = () => typeof globalThis.__wabou_motion_create === "function";
function fallbackSample(descriptor, elapsed) {
	if (descriptor.kind === "tween") {
		const raw = descriptor.duration === 0 ? 1 : Math.min(1, elapsed / descriptor.duration);
		return {
			value: raw,
			velocity: 0,
			done: raw >= 1
		};
	}
	const rate = Math.max(1, descriptor.damping / descriptor.mass);
	const value = 1 - Math.exp(-rate * elapsed);
	const done = 1 - value <= descriptor.restDelta;
	return {
		value: done ? 1 : value,
		velocity: 0,
		done
	};
}
var Curve = class {
	descriptor;
	id;
	#output = /* @__PURE__ */ new Float64Array(3);
	constructor(descriptor) {
		this.descriptor = descriptor;
		this.id = hasNativeMotion() ? globalThis.__wabou_motion_create(JSON.stringify(descriptor)) : void 0;
	}
	sample(elapsed) {
		if (this.id === void 0) return fallbackSample(this.descriptor, elapsed);
		if (!globalThis.__wabou_motion_sample(this.id, elapsed, this.#output)) throw new Error("motion handle is no longer live");
		return {
			value: this.#output[0],
			velocity: this.#output[1],
			done: this.#output[2] !== 0
		};
	}
	drop() {
		if (this.id !== void 0) globalThis.__wabou_motion_drop(this.id);
	}
};
function curveDescriptor(options) {
	if (options.type === "spring") return {
		kind: "spring",
		stiffness: options.stiffness ?? 170,
		damping: options.damping ?? 26,
		mass: options.mass ?? 1,
		velocity: options.velocity ?? 0,
		restDelta: options.restDelta ?? .001,
		restSpeed: options.restSpeed ?? .01
	};
	const ease = options.ease;
	return {
		kind: "tween",
		duration: options.type === false ? 0 : Math.max(0, options.duration ?? .3),
		easing: Array.isArray(ease) ? { cubicBezier: [...ease] } : typeof ease === "string" ? ease : "linear"
	};
}
function parseColor(value) {
	const hex = /^#([0-9a-f]{3,8})$/i.exec(value)?.[1];
	if (hex) {
		const full = hex.length === 3 || hex.length === 4 ? [...hex].map((part) => part + part).join("") : hex;
		if (full.length === 6 || full.length === 8) return [
			Number.parseInt(full.slice(0, 2), 16),
			Number.parseInt(full.slice(2, 4), 16),
			Number.parseInt(full.slice(4, 6), 16),
			full.length === 8 ? Number.parseInt(full.slice(6, 8), 16) / 255 : 1
		];
	}
	const rgb = /^rgba?\(\s*([\d.]+)[, ]+([\d.]+)[, ]+([\d.]+)(?:[, /]+([\d.]+))?\s*\)$/i.exec(value);
	return rgb ? [
		Number(rgb[1]),
		Number(rgb[2]),
		Number(rgb[3]),
		rgb[4] === void 0 ? 1 : Number(rgb[4])
	] : void 0;
}
const linearChannel = (channel) => {
	const value = channel / 255;
	return value * value;
};
const srgbChannel = (channel) => Math.max(0, Math.min(255, Math.round(255 * Math.sqrt(Math.max(0, channel)))));
function interpolateValue(from, to, progress) {
	if (typeof from === "number" && typeof to === "number") return from + (to - from) * progress;
	if (typeof from === "string" && typeof to === "string") {
		const a = parseColor(from);
		const b = parseColor(to);
		if (!a || !b) throw new TypeError("Wabou animations support numeric and CSS color values");
		const channels = [
			0,
			1,
			2
		].map((index) => srgbChannel(linearChannel(a[index]) + (linearChannel(b[index]) - linearChannel(a[index])) * progress));
		const alpha = Math.max(0, Math.min(1, a[3] + (b[3] - a[3]) * progress));
		return `rgba(${channels[0]}, ${channels[1]}, ${channels[2]}, ${Number(alpha.toFixed(4))})`;
	}
	throw new TypeError("animation keyframes must have one supported value type");
}
function interpolateKeyframes(keyframes, progress, times) {
	if (keyframes.length === 1) return keyframes[0];
	const points = times?.length === keyframes.length ? times : keyframes.map((_, index) => index / (keyframes.length - 1));
	let segment = points.length - 2;
	for (let index = 0; index < points.length - 1; index++) if (progress <= points[index + 1]) {
		segment = index;
		break;
	}
	const start = points[segment];
	const end = points[segment + 1];
	const local = end === start ? 1 : (progress - start) / (end - start);
	return interpolateValue(keyframes[segment], keyframes[segment + 1], local);
}
var Controls = class {
	keyframes;
	options;
	#curve;
	#state;
	#speed = 1;
	#frame;
	#lastTime;
	#elapsed = 0;
	#iteration = 0;
	#resolve;
	finished;
	constructor(keyframes, options) {
		this.keyframes = keyframes;
		this.options = options;
		this.#curve = new Curve(curveDescriptor(options));
		this.#state = options.autoplay === false ? "paused" : "running";
		this.finished = new Promise((resolve) => {
			this.#resolve = resolve;
		});
		options.onUpdate?.(keyframes[0]);
		if (this.#state === "running") {
			options.onPlay?.();
			this.#lastTime = performance.now();
			this.#frame = requestAnimationFrame(this.#tick);
		}
	}
	get speed() {
		return this.#speed;
	}
	set speed(value) {
		if (!Number.isFinite(value) || value <= 0) throw new RangeError("animation speed must be positive");
		this.#speed = value;
	}
	get duration() {
		return this.options.duration ?? (this.options.type === "spring" ? 0 : .3);
	}
	get state() {
		return this.#state;
	}
	#tick = (time) => {
		if (this.#state !== "running") return;
		const delta = this.#lastTime === void 0 ? 0 : Math.max(0, (time - this.#lastTime) / 1e3);
		this.#lastTime = time;
		this.#elapsed += delta * this.#speed;
		const delay = this.#iteration === 0 ? this.options.delay ?? 0 : this.options.repeatDelay ?? 0;
		if (this.#elapsed < delay) {
			this.#frame = requestAnimationFrame(this.#tick);
			return;
		}
		const sample = this.#curve.sample(this.#elapsed - delay);
		const progress = this.options.repeatType !== "loop" && this.#iteration % 2 === 1 ? 1 - sample.value : sample.value;
		this.options.onUpdate?.(interpolateKeyframes(this.keyframes, progress, this.options.times));
		if (!sample.done) {
			this.#frame = requestAnimationFrame(this.#tick);
			return;
		}
		const repeat = this.options.repeat ?? 0;
		if (repeat === Infinity || this.#iteration < repeat) {
			this.#iteration += 1;
			this.#elapsed = 0;
			this.#lastTime = time;
			this.options.onRepeat?.();
			this.#frame = requestAnimationFrame(this.#tick);
			return;
		}
		this.#finish();
	};
	#finish() {
		if (this.#state === "finished") return;
		const value = this.options.repeatType !== "loop" && this.#iteration % 2 === 1 ? this.keyframes[0] : this.keyframes[this.keyframes.length - 1];
		this.options.onUpdate?.(value);
		this.#state = "finished";
		this.#curve.drop();
		this.options.onComplete?.();
		this.#resolve();
	}
	play() {
		if (this.#state === "running" || this.#state === "finished") return;
		this.#state = "running";
		this.#lastTime = performance.now();
		this.options.onPlay?.();
		this.#frame = requestAnimationFrame(this.#tick);
	}
	pause() {
		if (this.#state !== "running") return;
		if (this.#frame !== void 0) cancelAnimationFrame(this.#frame);
		this.#frame = void 0;
		this.#lastTime = void 0;
		this.#state = "paused";
	}
	stop() {
		if (this.#state === "idle" || this.#state === "finished") return;
		if (this.#frame !== void 0) cancelAnimationFrame(this.#frame);
		this.#frame = void 0;
		this.#state = "idle";
		this.#curve.drop();
		this.options.onStop?.();
		this.#resolve();
	}
	cancel() {
		this.stop();
	}
	complete() {
		if (this.#frame !== void 0) cancelAnimationFrame(this.#frame);
		this.#frame = void 0;
		this.#finish();
	}
	then(onfulfilled, onrejected) {
		return this.finished.then(onfulfilled, onrejected);
	}
};
/** Animate between two supported values using a Rust-owned curve. */
function animate(from, to, options = {}) {
	return animateKeyframes([from, to], options);
}
/** Animate numeric or color keyframes using a Rust-owned curve. */
function animateKeyframes(keyframes, options = {}) {
	return new Controls(keyframes, options);
}
const read = (value, fallback) => typeof value === "function" ? value() : value ?? fallback;
function positiveFinite(value, fallback) {
	return Number.isFinite(value) && value > 0 ? value : fallback;
}
function createNativeLoopAnimation(options = {}) {
	return createMemo(() => ({
		kind: "loop",
		duration: positiveFinite(read(options.duration, 1), 1),
		speed: positiveFinite(read(options.speed, 1), 1),
		paused: read(options.paused, false),
		reducedMotion: read(options.reducedMotion, false)
	}));
}
function createKeyframeAnimation(keyframes, options = {}) {
	const { reducedMotion, reducedValue = keyframes[keyframes.length - 1], onUpdate, ...animationOptions } = options;
	const authoredAutoplay = animationOptions.autoplay ?? true;
	const initiallyReduced = untrack(() => read(reducedMotion, false));
	const [box, setBox] = createSignal({ value: initiallyReduced ? reducedValue : keyframes[0] });
	const value = () => box().value;
	const controls = animateKeyframes(keyframes, {
		...animationOptions,
		autoplay: authoredAutoplay && !initiallyReduced,
		onUpdate(next) {
			setBox({ value: next });
			onUpdate?.(next);
		}
	});
	let initialized = false;
	let resumeAfterReduction = authoredAutoplay;
	createEffect(() => read(reducedMotion, false), (reduced) => {
		if (reduced) {
			if (initialized) resumeAfterReduction = controls.state === "running";
			controls.pause();
			setBox({ value: reducedValue });
			onUpdate?.(reducedValue);
		} else if (initialized && resumeAfterReduction) controls.play();
		initialized = true;
	});
	onCleanup(() => controls.stop());
	return {
		value,
		controls
	};
}
function createInterpolation(source, input, output, options = {}) {
	if (input.length === 0 || input.length !== output.length) throw new RangeError("animation input and output ranges must have equal non-zero lengths");
	if (!input.every(Number.isFinite)) throw new RangeError("animation input range must contain only finite numbers");
	return () => {
		let value = source();
		if (options.clamp !== false) value = Math.max(input[0], Math.min(input[input.length - 1], value));
		let segment = input.length - 2;
		for (let index = 0; index < input.length - 1; index++) if (value <= input[index + 1]) {
			segment = index;
			break;
		}
		const progress = (value - input[segment]) / (input[segment + 1] - input[segment]);
		return interpolateValue(output[segment], output[segment + 1], progress);
	};
}
/**
* Lifecycle-owned scalar transition that retargets from its current value.
*
* Unlike a one-shot animation, changing `target` while a run is active does
* not restart from the previous keyframe. This makes it suitable for rapidly
* toggled disclosure, hover and selection state.
*/
function createTransition(target, options = {}) {
	const [value, setValue] = createSignal(untrack(() => read(options.initial, target())), { ownedWrite: true });
	const [state, setState] = createSignal("idle", { ownedWrite: true });
	let controls;
	let generation = 0;
	const stop = () => {
		generation++;
		controls?.stop();
		controls = void 0;
		setState("idle");
	};
	const jump = (next) => {
		stop();
		setValue(next);
		options.onUpdate?.(next);
		options.onComplete?.(next);
	};
	createEffect(() => [
		target(),
		read(options.reducedMotion, false),
		read(options.duration, .3),
		untrack(value)
	], ([next, reduced, duration, current]) => {
		if (Object.is(next, current)) return;
		if (reduced || options.type === false || duration === 0) {
			jump(next);
			return;
		}
		const run = ++generation;
		controls?.stop();
		setState("running");
		const { initial: _initial, duration: _duration, reducedMotion: _reducedMotion, onUpdate, onComplete, ...animationOptions } = options;
		controls = animate(current, next, {
			...animationOptions,
			duration,
			onUpdate(current) {
				if (run !== generation) return;
				setValue(current);
				onUpdate?.(current);
			},
			onComplete() {
				if (run !== generation) return;
				controls = void 0;
				setValue(next);
				setState("finished");
				onComplete?.(next);
			}
		});
	});
	onCleanup(stop);
	return {
		value,
		state,
		jump,
		stop
	};
}
function createRepeatingAnimation(keyframes, options) {
	return createKeyframeAnimation(keyframes, options);
}
/**
* Lifecycle-owned repeating scalar animation for Solid components.
*
* The controls stop automatically with the current Solid owner.
*/
function createLoop(options = {}) {
	const from = options.from ?? 0;
	const to = options.to ?? 1;
	const { from: _from, to: _to, reducedMotion, reducedValue = from, onUpdate, ...animationOptions } = options;
	return createRepeatingAnimation([from, to], {
		duration: 1,
		ease: "linear",
		repeat: Infinity,
		...animationOptions,
		reducedMotion,
		reducedValue,
		onUpdate
	});
}
function normalizeSweepGeometry(extent, itemRatio) {
	return {
		extent: Number.isFinite(extent) ? Math.max(0, extent) : 0,
		itemRatio: Number.isFinite(itemRatio) && itemRatio > 0 ? Math.min(1, itemRatio) : .4
	};
}
/**
* Move an item completely across one measured axis using only a runtime
* transform. Both repeat boundaries remain outside the container, avoiding a
* visible reset and avoiding per-frame layout invalidation.
*/
function createSweep(options) {
	const { extent, itemRatio, axis = "horizontal", ...loopOptions } = options;
	const loop = createLoop(loopOptions);
	const geometry = () => normalizeSweepGeometry(read(extent, 0), read(itemRatio, .4));
	const offset = () => {
		const current = geometry();
		return current.extent * (loop.value() * (1 + current.itemRatio) - current.itemRatio);
	};
	return {
		...loop,
		offset,
		transform: () => axis === "vertical" ? translate2d(0, offset()) : translate2d(offset(), 0)
	};
}
/** Repeating center-pivoted rotation backed by Motion value animation. */
function createRotation(options = {}) {
	const loop = createLoop({
		...options,
		from: options.from ?? 0,
		to: options.to ?? Math.PI * 2
	});
	return {
		...loop,
		angle: loop.value,
		transform: () => rotate2d(loop.value())
	};
}
/** Repeating from→to→from value animation with automatic cleanup. */
function createPulse(options = {}) {
	const from = options.from ?? .5;
	const to = options.to ?? 1;
	const { from: _from, to: _to, reducedMotion, reducedValue = to, onUpdate, ...animationOptions } = options;
	return createRepeatingAnimation([
		from,
		to,
		from
	], {
		duration: 1,
		ease: "easeInOut",
		repeat: Infinity,
		...animationOptions,
		reducedMotion,
		reducedValue,
		onUpdate
	});
}
//#endregion
export { createLoop as a, createRotation as c, normalizeSweepGeometry as d, useSpring as f, useReducedMotion as g, useMotionConfig as h, createKeyframeAnimation as i, createSweep as l, MotionConfigProvider as m, animateKeyframes as n, createNativeLoopAnimation as o, useTween as p, createInterpolation as r, createPulse as s, animate as t, createTransition as u };

//# sourceMappingURL=animation-CsAC1wRH.mjs.map