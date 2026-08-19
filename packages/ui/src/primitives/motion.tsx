import { scale2d } from "@wabou/core/style";
import { createEffect, type JSX, omit } from "solid-js";
import {
  type AnimationControls,
  createLoop,
  createPulse,
  createRotation,
} from "../animation";
import { View, type ViewProps, type WabouStyle } from "./view";

interface PlaybackProps {
  /** Animation duration in seconds. */
  duration?: number;
  /** Playback-rate multiplier. */
  speed?: number;
  paused?: boolean;
}

function bindPlayback(
  controls: AnimationControls,
  props: Pick<PlaybackProps, "paused" | "speed">,
) {
  createEffect(
    () => props.speed ?? 1,
    (speed) => {
      controls.speed = speed;
    },
  );
  createEffect(
    () => props.paused,
    (paused) => {
      if (paused) controls.pause();
      else controls.play();
    },
  );
}

export interface SpinProps extends Omit<ViewProps, "transform">, PlaybackProps {
  children?: JSX.Element;
}

/** A single native View whose contents rotate around its border-box center. */
export function Spin(props: SpinProps): JSX.Element {
  const motion = props;
  const view = omit(props, "duration", "speed", "paused");
  const rotation = createRotation({
    autoplay: !motion.paused,
    duration: motion.duration ?? 1,
  });
  bindPlayback(rotation.controls, motion);
  return <View {...view} transform={rotation.transform()} />;
}

export interface PulseProps extends ViewProps, PlaybackProps {
  from?: number;
  to?: number;
}

/** A single native View with a repeating opacity pulse. */
export function Pulse(props: PulseProps): JSX.Element {
  const motion = props;
  const view = omit(
    props,
    "duration",
    "speed",
    "paused",
    "from",
    "to",
    "style",
  );
  const pulse = createPulse({
    autoplay: !motion.paused,
    duration: motion.duration ?? 1,
    from: motion.from,
    to: motion.to,
  });
  bindPlayback(pulse.controls, motion);
  const style = (): WabouStyle => ({
    ...(motion.style ?? {}),
    opacity: pulse.value(),
  });
  return <View {...view} style={style()} />;
}

export interface RippleProps extends ViewProps, PlaybackProps {
  /** Scale at the beginning of each ripple. Defaults to 0.35. */
  fromScale?: number;
}

/** A center-originating ring that expands while fading out, then repeats. */
export function Ripple(props: RippleProps): JSX.Element {
  const motion = props;
  const view = omit(
    props,
    "duration",
    "speed",
    "paused",
    "fromScale",
    "style",
    "transform",
  );
  const ripple = createLoop({
    autoplay: !motion.paused,
    duration: motion.duration ?? 1.4,
    from: 0,
    to: 1,
  });
  bindPlayback(ripple.controls, motion);
  const progress = () => ripple.value();
  return (
    <View
      {...view}
      transform={scale2d(
        (motion.fromScale ?? 0.35) +
          progress() * (1 - (motion.fromScale ?? 0.35)),
      )}
      style={{ ...(motion.style ?? {}), opacity: 1 - progress() }}
    />
  );
}
