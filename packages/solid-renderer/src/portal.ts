import { omit, onCleanup, type JSX } from "solid-js";
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
  const local = props;
  const containerProps = omit(props, "children", "plane");
  const plane = local.plane ?? "floating";
  const root = acquireOverlayRoot(plane);
  const container = createElement("view") as Handle;
  spread(container, containerProps, false);
  if (plane === "modal") {
    // Mark each modal container, not only the shared plane root. The host can
    // then expose only the last painted modal when several are mounted.
    spread(container, { "aria-modal": "true", overlayPlane: "modal" }, false);
  }
  insertNode(root, container, undefined);
  insert(container, () => local.children);
  onCleanup(() => {
    if (container.parent) removeNode(container.parent, container);
    releaseOverlayRoot(plane);
  });
  return null;
}
