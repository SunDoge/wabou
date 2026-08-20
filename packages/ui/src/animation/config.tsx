import { type Accessor, createContext, type JSX, useContext } from "solid-js";

export interface MotionConfig {
  reducedMotion: Accessor<boolean>;
}

const DEFAULT_MOTION_CONFIG: MotionConfig = Object.freeze({
  reducedMotion: () => false,
});

const MotionConfigContext = createContext<MotionConfig>(DEFAULT_MOTION_CONFIG);

export interface MotionConfigProviderProps {
  /** Disable non-essential interpolation while preserving final UI state. */
  reducedMotion?: boolean;
  children?: JSX.Element;
}

/** Application-level motion policy inherited by all styled Wabou components. */
export function MotionConfigProvider(
  props: MotionConfigProviderProps,
): JSX.Element {
  const parent = useContext(MotionConfigContext);
  const value: MotionConfig = {
    reducedMotion: () => props.reducedMotion ?? parent.reducedMotion(),
  };
  return (
    <MotionConfigContext value={value}>{props.children}</MotionConfigContext>
  );
}

export function useMotionConfig(): MotionConfig {
  return useContext(MotionConfigContext);
}

export function useReducedMotion(): Accessor<boolean> {
  return useMotionConfig().reducedMotion;
}
