import type { ImageViewportSize } from "@wabou/ui";
import type { OcrRegion } from "./api";

export type RegionGeometryProperty = "x" | "y" | "width" | "height";

export interface RegionRevealTarget {
  focus(): void;
}

export function selectRegionAndReveal(
  id: string | null,
  select: (id: string | null) => void,
  targets: ReadonlyMap<string, RegionRevealTarget>,
): void {
  select(id);
  if (id !== null) targets.get(id)?.focus();
}

export function updateRegionGeometry(
  region: OcrRegion,
  property: RegionGeometryProperty,
  value: number,
  image: ImageViewportSize,
): OcrRegion {
  const next = { ...region, [property]: value };
  const x = Math.max(0, Math.min(image.width - 1, next.x));
  const y = Math.max(0, Math.min(image.height - 1, next.y));
  return {
    ...next,
    x,
    y,
    width: Math.max(1, Math.min(image.width - x, next.width)),
    height: Math.max(1, Math.min(image.height - y, next.height)),
  };
}

export function translatedRegions(
  regions: readonly OcrRegion[],
): readonly OcrRegion[] {
  return regions.filter((region) => Boolean(region.translation?.trim()));
}
