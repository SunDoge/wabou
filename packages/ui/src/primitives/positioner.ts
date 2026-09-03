import type {
  Handle,
  WabouFloatingPlacement,
  WabouFloatingPosition,
} from "@wabou/core/renderer";

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
export function floatingFromNode(
  anchor: Handle,
  options: FloatingPositionOptions = {},
): WabouFloatingPosition {
  return {
    anchor: { kind: "node", id: anchor.id },
    placement: options.placement,
    offset: options.offset,
    margin: options.margin,
  };
}

/** Build the explicit native positioning contract for a viewport point. */
export function floatingFromPoint(
  point: PointAnchor,
  options: FloatingPositionOptions = {},
): WabouFloatingPosition {
  if (![point.x, point.y].every(Number.isFinite)) {
    throw new RangeError("floating point anchor must be finite");
  }
  return {
    anchor: { kind: "point", x: point.x, y: point.y },
    placement: options.placement,
    offset: options.offset,
    margin: options.margin,
  };
}
