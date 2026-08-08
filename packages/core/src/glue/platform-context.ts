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
  return createComponent(PlatformContext.Provider, {
    get value() {
      return props.value;
    },
    get children() {
      return props.children;
    },
  });
}

export function usePlatformServices(): Partial<PlatformServices> {
  return useContext(PlatformContext) ?? {};
}
