import { onCleanup, splitProps, type JSX } from "solid-js";
import {
  createElement,
  getMountRoot,
  insert,
  insertNode,
  removeNode,
  spread,
  type Handle,
} from "./index";

export interface PortalProps {
  children?: JSX.Element;
  /** Host stacking plane. `system` and `debug` are reserved for native UI. */
  plane?: "floating" | "modal";
  [name: string]: unknown;
}

/** Render a native host subtree directly under the mounted window root. */
export function Portal(props: PortalProps): JSX.Element {
  const [local, containerProps] = splitProps(props, ["children", "plane"]);
  const root = getMountRoot();
  const container = createElement("view") as Handle;
  spread(container, containerProps, false);
  spread(container, { get overlayPlane() { return local.plane ?? "floating"; } }, false);
  insertNode(root, container, undefined);
  insert(container, () => local.children);
  onCleanup(() => {
    if (container.parent) removeNode(container.parent, container);
  });
  return null;
}
