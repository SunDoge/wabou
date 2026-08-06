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
  [name: string]: unknown;
}

/** Render a native host subtree directly under the mounted window root. */
export function Portal(props: PortalProps): JSX.Element {
  const [local, containerProps] = splitProps(props, ["children"]);
  const root = getMountRoot();
  const container = createElement("view") as Handle;
  spread(container, containerProps, false);
  insertNode(root, container, undefined);
  insert(container, () => local.children);
  onCleanup(() => {
    if (container.parent) removeNode(container.parent, container);
  });
  return null;
}
