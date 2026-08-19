import { PathBuilder } from "@wabou/ui";

export function smoothPath(
  values: readonly number[],
  width: number,
  height: number,
) {
  const points = values.map((value, index) => ({
    x: values.length <= 1 ? 0 : (index / (values.length - 1)) * width,
    y: height - value * height,
  }));
  const path = new PathBuilder();
  if (!points.length) return path;
  path.moveTo(points[0].x, points[0].y);
  for (let index = 0; index < points.length - 1; index++) {
    const before = points[Math.max(0, index - 1)];
    const start = points[index];
    const end = points[index + 1];
    const after = points[Math.min(points.length - 1, index + 2)];
    path.cubicTo(
      start.x + (end.x - before.x) / 6,
      start.y + (end.y - before.y) / 6,
      end.x - (after.x - start.x) / 6,
      end.y - (after.y - start.y) / 6,
      end.x,
      end.y,
    );
  }
  return path;
}
