import { ViewProps } from "./view.mjs";
import { JSX } from "solid-js";
//#region src/primitives/motion.d.ts
interface PlaybackProps {
  /** Animation duration in seconds. */
  duration?: number;
  /** Playback-rate multiplier. */
  speed?: number;
  paused?: boolean;
}
export interface SpinProps extends Omit<ViewProps, "transform">, PlaybackProps {
  children?: JSX.Element;
}
/** A single native View whose contents rotate around its border-box center. */
export declare function Spin(props: SpinProps): JSX.Element;
export interface PulseProps extends ViewProps, PlaybackProps {
  from?: number;
  to?: number;
}
/** A single native View with a repeating opacity pulse. */
export declare function Pulse(props: PulseProps): JSX.Element;
export interface RippleProps extends ViewProps, PlaybackProps {
  /** Scale at the beginning of each ripple. Defaults to 0.35. */
  fromScale?: number;
}
/** A center-originating ring that expands while fading out, then repeats. */
export declare function Ripple(props: RippleProps): JSX.Element;
//#endregion
export {};
//# sourceMappingURL=motion.d.mts.map