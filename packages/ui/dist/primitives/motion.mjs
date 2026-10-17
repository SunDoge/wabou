import { useReducedMotion } from "../animation/config.mjs";
import { createLoop, createPulse, createRotation } from "../animation.mjs";
import { View } from "./view.mjs";
import { scale2d } from "@wabou/core/style";
import { createEffect, omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/primitives/motion.tsx
function bindPlayback(controls, props, reducedMotion) {
	createEffect(() => props.speed ?? 1, (speed) => {
		controls.speed = speed;
	});
	createEffect(() => (props.paused ?? false) || reducedMotion(), (stopped) => {
		if (stopped) controls.pause();
		else controls.play();
	});
}
/** A single native View whose contents rotate around its border-box center. */
function Spin(props) {
	const motion = props;
	const reducedMotion = useReducedMotion();
	const view = omit(props, "duration", "speed", "paused");
	const rotation = createRotation({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1,
		reducedMotion,
		reducedValue: 0
	});
	bindPlayback(rotation.controls, motion, reducedMotion);
	return createComponent$1(View, mergeProps(view, { get transform() {
		return rotation.transform();
	} }));
}
/** A single native View with a repeating opacity pulse. */
function Pulse(props) {
	const motion = props;
	const reducedMotion = useReducedMotion();
	const view = omit(props, "duration", "speed", "paused", "from", "to", "style");
	const pulse = createPulse({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1,
		from: motion.from,
		to: motion.to,
		reducedMotion,
		reducedValue: motion.to ?? 1
	});
	bindPlayback(pulse.controls, motion, reducedMotion);
	const style = () => ({
		...motion.style ?? {},
		opacity: pulse.value()
	});
	return createComponent$1(View, mergeProps(view, { get style() {
		return style();
	} }));
}
/** A center-originating ring that expands while fading out, then repeats. */
function Ripple(props) {
	const motion = props;
	const reducedMotion = useReducedMotion();
	const view = omit(props, "duration", "speed", "paused", "fromScale", "style", "transform");
	const ripple = createLoop({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1.4,
		from: 0,
		to: 1,
		reducedMotion,
		reducedValue: 1
	});
	bindPlayback(ripple.controls, motion, reducedMotion);
	const progress = () => ripple.value();
	return createComponent$1(View, mergeProps(view, {
		get transform() {
			return scale2d((motion.fromScale ?? .35) + progress() * (1 - (motion.fromScale ?? .35)));
		},
		get style() {
			return {
				...motion.style ?? {},
				opacity: reducedMotion() ? 0 : 1 - progress()
			};
		}
	}));
}
//#endregion
export { Pulse, Ripple, Spin };

//# sourceMappingURL=motion.mjs.map