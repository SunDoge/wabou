import { Accessor } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/primitives/measure.d.ts
export interface MeasuredSize {
  ref(node: Handle): void;
  width: Accessor<number>;
  height: Accessor<number>;
  measured: Accessor<boolean>;
}
export interface MeasuredSizeOptions {
  onChange?: (size: {
    width: number;
    height: number;
  }) => void;
}
/** Inclusive logical-pixel constraints evaluated against a host node's content box. */
export interface ContainerSizeQuery {
  minWidth?: number;
  maxWidth?: number;
  minHeight?: number;
  maxHeight?: number;
}
export interface ContainerMatch extends MeasuredSize {
  matches: Accessor<boolean>;
}
/** Observe the completed native content-box size of a host node. */
export declare function createMeasuredSize(options?: MeasuredSizeOptions): MeasuredSize;
/**
 * Match constraints against a component's completed native content-box size.
 * The result remains false until the first measurement, avoiding a compact
 * layout flash during boot.
 */
export declare function createContainerMatch(query: ContainerSizeQuery, options?: MeasuredSizeOptions): ContainerMatch;
//#endregion
//# sourceMappingURL=measure.d.mts.map