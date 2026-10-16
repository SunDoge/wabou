import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { FileDropPosition } from "@wabou/core";
import { JSX } from "solid-js";
import { LayoutRect } from "@wabou/core/renderer";
//#region src/components/drop-zone.d.ts
export interface DropZoneProps extends Omit<ViewProps, "class" | "children"> {
  /** Called with paths accepted by this zone after a native drop. */
  onDrop: (paths: readonly string[]) => void;
  /** Return true for paths this zone accepts. All paths are accepted by default. */
  accept?: (path: string) => boolean;
  /** Reports paths rejected by `accept` without hiding a partially valid drop. */
  onRejected?: (paths: readonly string[]) => void;
  disabled?: boolean;
  label?: string;
  activeLabel?: string;
  description?: string;
  class?: string;
}
/** Inclusive hit test in the logical coordinate space shared with native DnD. */
export declare function pointInLayoutRect(point: FileDropPosition, rect: LayoutRect): boolean;
/**
 * A component-local target for native filesystem drops.
 *
 * Native backends publish window-relative coordinates, so the zone measures
 * its completed native layout before claiming an event. Events without a
 * position are deliberately ignored instead of being delivered ambiguously to
 * every mounted zone.
 */
export declare function DropZone(props: DropZoneProps): JSX.Element;
//#endregion
//# sourceMappingURL=drop-zone.d.mts.map