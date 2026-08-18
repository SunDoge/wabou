import "@wabou/core";
import { rotate2d } from "@wabou/core/style";
import { animateValue } from "motion-dom";
import { createEffect, createSignal, onCleanup, untrack } from "solid-js";
//#region src/index.ts
var Controls = class {
	backend;
	constructor(backend) {
		this.backend = backend;
	}
	get time() {
		return this.backend.time;
	}
	set time(value) {
		this.backend.time = value;
	}
	get speed() {
		return this.backend.speed;
	}
	set speed(value) {
		this.backend.speed = value;
	}
	get duration() {
		return this.backend.duration;
	}
	get state() {
		const state = this.backend.state;
		if (state === "running" || state === "paused" || state === "finished") return state;
		return "idle";
	}
	get finished() {
		return this.backend.finished.then(() => void 0);
	}
	play() {
		this.backend.play();
	}
	pause() {
		this.backend.pause();
	}
	stop() {
		this.backend.stop();
	}
	cancel() {
		this.backend.cancel();
	}
	complete() {
		this.backend.complete();
	}
	then(onfulfilled, onrejected) {
		return this.finished.then(onfulfilled, onrejected);
	}
};
function normalizeOptions(options) {
	const normalized = { ...options };
	for (const key of [
		"delay",
		"duration",
		"repeatDelay"
	]) {
		const seconds = options[key];
		if (typeof seconds === "number") normalized[key] = seconds * 1e3;
	}
	if (typeof options.visualDuration === "number") normalized.visualDuration = options.visualDuration * 1e3;
	return normalized;
}
/** Animate between two scalar values. */
function animate(from, to, options = {}) {
	return animateKeyframes([from, to], options);
}
/** Animate through two or more scalar keyframes. */
function animateKeyframes(keyframes, options = {}) {
	return new Controls(animateValue({
		...normalizeOptions(options),
		keyframes: [...keyframes]
	}));
}
const read = (value, fallback) => typeof value === "function" ? value() : value ?? fallback;
/**
* Lifecycle-owned scalar transition that retargets from its current value.
*
* Unlike a one-shot animation, changing `target` while a run is active does
* not restart from the previous keyframe. This makes it suitable for rapidly
* toggled disclosure, hover and selection state.
*/
function createTransition(target, options = {}) {
	const [value, setValue] = createSignal(target());
	const [state, setState] = createSignal("idle");
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
		untrack(value)
	], ([next, reduced, current]) => {
		if (Object.is(next, current)) return;
		if (reduced || options.type === false || options.duration === 0) {
			jump(next);
			return;
		}
		const run = ++generation;
		controls?.stop();
		setState("running");
		const { reducedMotion: _reducedMotion, onUpdate, onComplete, ...animationOptions } = options;
		controls = animate(current, next, {
			...animationOptions,
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
/**
* Lifecycle-owned repeating scalar animation for Solid components.
*
* The controls stop automatically with the current Solid owner.
*/
function createLoop(options = {}) {
	const from = options.from ?? 0;
	const to = options.to ?? 1;
	const { from: _from, to: _to, onUpdate, ...animationOptions } = options;
	const [value, setValue] = createSignal(from);
	const controls = animate(from, to, {
		duration: 1,
		ease: "linear",
		repeat: Infinity,
		...animationOptions,
		onUpdate(next) {
			setValue(next);
			onUpdate?.(next);
		}
	});
	onCleanup(() => controls.stop());
	return {
		value,
		controls
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
	const { from: _from, to: _to, onUpdate, ...animationOptions } = options;
	const [value, setValue] = createSignal(from);
	const controls = animateKeyframes([
		from,
		to,
		from
	], {
		duration: 1,
		ease: "easeInOut",
		repeat: Infinity,
		...animationOptions,
		onUpdate(next) {
			setValue(next);
			onUpdate?.(next);
		}
	});
	onCleanup(() => controls.stop());
	return {
		value,
		controls
	};
}
//#endregion
export { animate, animateKeyframes, createLoop, createPulse, createRotation, createTransition };

//# sourceMappingURL=index.mjs.map