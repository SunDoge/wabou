import { createComponent, createContext, type JSX, useContext } from "solid-js";
import type { Clipboard } from "./clipboard";
import type { WindowState } from "./window-metrics";

export interface PlatformServices {
  clipboard: Clipboard;
  window: WindowState;
}

const PlatformContext = createContext<Partial<PlatformServices>>();

export interface PlatformProviderProps {
  value: Partial<PlatformServices>;
  children?: JSX.Element;
}

/** Override native services for one Solid subtree, primarily for tests and previews. */
export function PlatformProvider(props: PlatformProviderProps): JSX.Element {
  const parent = useContext(PlatformContext) ?? {};
  const value: Partial<PlatformServices> = {
    get clipboard() {
      return props.value.clipboard ?? parent.clipboard;
    },
    get window() {
      return props.value.window ?? parent.window;
    },
  };
  return createComponent(PlatformContext.Provider, {
    value,
    get children() {
      return props.children;
    },
  });
}

export function usePlatformServices(): Partial<PlatformServices> {
  return useContext(PlatformContext) ?? {};
}
