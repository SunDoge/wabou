import { animate, animateKeyframes, createTransition } from "@wabou/animation";
import { createRoot, createSignal } from "solid-js";

const result = {
  done: false,
  number: 0,
  color: "",
  spring: 0,
  keyframe: 0,
  cancelled: 0,
  transition: 0,
  transitionPeak: 0,
  transitionDone: false,
};

Object.assign(globalThis, { __wabou_motion_result: result });

const cancelled = animate(0, 100, {
  duration: 10,
  onUpdate: (value) => (result.cancelled = value),
});
cancelled.stop();

const animations = [
  animate(0, 100, {
    duration: 0.04,
    ease: "linear",
    onUpdate: (value) => (result.number = value),
  }),
  animate("#000000", "#ffffff", {
    duration: 0.04,
    ease: "linear",
    onUpdate: (value) => (result.color = value),
  }),
  animate(0, 1, {
    type: "spring",
    stiffness: 500,
    damping: 35,
    onUpdate: (value) => (result.spring = value),
  }),
  animateKeyframes([0, 50, 10], {
    duration: 0.04,
    times: [0, 0.5, 1],
    ease: "linear",
    onUpdate: (value) => (result.keyframe = value),
  }),
];

Promise.all(
  animations.map((animation) => animation.then(() => undefined)),
).then(() => {
  result.done = result.transitionDone;
});

createRoot(() => {
  const [target, setTarget] = createSignal(0, { ownedWrite: true });
  createTransition(target, {
    duration: 0.08,
    ease: "linear",
    onUpdate(value) {
      result.transition = value;
      result.transitionPeak = Math.max(result.transitionPeak, value);
    },
    onComplete(value) {
      if (value !== 20) return;
      result.transitionDone = true;
      result.done = animations.every(
        (animation) => animation.state === "finished",
      );
    },
  });
  setTarget(100);
  requestAnimationFrame(() => setTarget(20));
});
