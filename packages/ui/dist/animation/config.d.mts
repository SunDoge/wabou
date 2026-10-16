import { Accessor, JSX } from "solid-js";
//#region src/animation/config.d.ts
export interface MotionConfig {
  reducedMotion: Accessor<boolean>;
}
export interface MotionConfigProviderProps {
  /** Disable non-essential interpolation while preserving final UI state. */
  reducedMotion?: boolean;
  children?: JSX.Element;
}
/** Application-level motion policy inherited by all styled Wabou components. */
export declare function MotionConfigProvider(props: MotionConfigProviderProps): JSX.Element;
export declare function useMotionConfig(): MotionConfig;
export declare function useReducedMotion(): Accessor<boolean>;
//#endregion
//# sourceMappingURL=config.d.mts.map