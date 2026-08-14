import {
  createPulse,
  createRotation,
  type AnimationControls,
} from "@wabou/animation";
import { createEffect, omit, type JSX } from "solid-js";
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
  createEffect(() => {
    controls.speed = props.speed ?? 1;
  });
  createEffect(() => {
    if (props.paused) controls.pause();
    else controls.play();
  });
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
