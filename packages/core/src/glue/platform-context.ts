import {
  createComponent,
  createContext,
  getOwner,
  type JSX,
  useContext,
} from "solid-js";
import type { Clipboard } from "./clipboard";
import type { Dialog } from "./dialog";
import type { Notification } from "./notification";
import type { WindowState } from "./window-metrics";

export interface PlatformServices {
  clipboard: Clipboard;
  dialog: Dialog;
  notification: Notification;
  window: WindowState;
}

const PlatformContext = createContext<Partial<PlatformServices>>({});

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
    get dialog() {
      return props.value.dialog ?? parent.dialog;
    },
    get notification() {
      return props.value.notification ?? parent.notification;
    },
    get window() {
      return props.value.window ?? parent.window;
    },
  };
  return createComponent(PlatformContext, {
    value,
    get children() {
      return props.children;
    },
  });
}

export function usePlatformServices(): Partial<PlatformServices> {
  return getOwner() ? useContext(PlatformContext) : {};
}
