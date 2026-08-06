import { onCleanup, splitProps, type JSX } from "solid-js";
import {
  createElement,
  acquireOverlayRoot,
  insert,
  insertNode,
  removeNode,
  releaseOverlayRoot,
  spread,
  type Handle,
} from "./index";

export interface PortalProps {
  children?: JSX.Element;
  /** Host stacking plane. `system` and `debug` are reserved for native UI. */
  plane?: "floating" | "modal";
  [name: string]: unknown;
}

/** Render a native host subtree under its shared synthetic overlay root. */
export function Portal(props: PortalProps): JSX.Element {
  const [local, containerProps] = splitProps(props, ["children", "plane"]);
  const plane = local.plane ?? "floating";
  const root = acquireOverlayRoot(plane);
  const container = createElement("view") as Handle;
  spread(container, containerProps, false);
  insertNode(root, container, undefined);
  insert(container, () => local.children);
  onCleanup(() => {
    if (container.parent) removeNode(container.parent, container);
    releaseOverlayRoot(plane);
  });
  return null;
}
