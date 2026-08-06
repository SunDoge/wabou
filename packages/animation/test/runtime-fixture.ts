import { animate, animateKeyframes } from "@wabou/animation";

const result = {
  done: false,
  number: 0,
  color: "",
  spring: 0,
  keyframe: 0,
  cancelled: 0,
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
  result.done = true;
});
