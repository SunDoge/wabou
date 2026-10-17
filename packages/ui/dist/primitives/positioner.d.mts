import { Handle, WabouFloatingPlacement, WabouFloatingPosition } from "@wabou/core/renderer";
//#region src/primitives/positioner.d.ts
export type Placement = WabouFloatingPlacement;
export interface PointAnchor {
  x: number;
  y: number;
}
export interface FloatingPositionOptions {
  placement?: Placement;
  offset?: number;
  margin?: number;
}
/** Build the explicit native positioning contract for a retained trigger. */
export declare function floatingFromNode(anchor: Handle, options?: FloatingPositionOptions): WabouFloatingPosition;
/** Build the explicit native positioning contract for a viewport point. */
export declare function floatingFromPoint(point: PointAnchor, options?: FloatingPositionOptions): WabouFloatingPosition;
//#endregion
//# sourceMappingURL=positioner.d.mts.map