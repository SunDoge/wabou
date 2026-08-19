import { PathBuilder } from "@wabou/ui";

export interface PathPoint {
  x: number;
  y: number;
}

/** Append a Catmull-Rom spline converted to cubic Bézier segments. */
export function appendSmoothPath(
  path: PathBuilder,
  points: readonly PathPoint[],
  tension = 1,
): PathBuilder {
  if (points.length === 0) return path;
  path.moveTo(points[0].x, points[0].y);
  if (points.length === 1) return path;
  const scale = tension / 6;
  for (let index = 0; index < points.length - 1; index++) {
    const before = points[Math.max(0, index - 1)];
    const start = points[index];
    const end = points[index + 1];
    const after = points[Math.min(points.length - 1, index + 2)];
    path.cubicTo(
      start.x + (end.x - before.x) * scale,
      start.y + (end.y - before.y) * scale,
      end.x - (after.x - start.x) * scale,
      end.y - (after.y - start.y) * scale,
      end.x,
      end.y,
    );
  }
  return path;
}
