export {
  INLINE_STYLE_CONTRACT,
  type WabouStyle,
} from "./generated/style-properties.ts";
export * from "./generated/utility-types.ts";

import { INLINE_STYLE_CONTRACT } from "./generated/style-properties.ts";
import type { WabouUtility } from "./generated/utility-types.ts";

export const STYLE_VALUE = "__wabou_style_value__" as const;

export const StyleValueKind = {
  Px: 1,
  Percent: 2,
  Number: 3,
  Boolean: 4,
  Color: 5,
  Auto: 6,
} as const;

export type StyleValueKind =
  (typeof StyleValueKind)[keyof typeof StyleValueKind];

export interface TypedStyleValue<K extends StyleValueKind = StyleValueKind> {
  readonly [STYLE_VALUE]: true;
  readonly kind: K;
  readonly value: number;
}

function finite(value: number, name: string): number {
  if (!Number.isFinite(value)) throw new TypeError(`${name} must be finite`);
  return value;
}

function typed<K extends StyleValueKind>(
  kind: K,
  value = 0,
): TypedStyleValue<K> {
  return { [STYLE_VALUE]: true, kind, value };
}

/** Device-independent logical pixels. */
export const px = (value: number): TypedStyleValue<typeof StyleValueKind.Px> =>
  typed(StyleValueKind.Px, finite(value, "px"));

/** A ratio where `1` means 100%. */
export const percent = (
  value: number,
): TypedStyleValue<typeof StyleValueKind.Percent> =>
  typed(StyleValueKind.Percent, finite(value, "percent"));

/** A unitless numeric style value such as opacity or flex-grow. */
export const number = (
  value: number,
): TypedStyleValue<typeof StyleValueKind.Number> =>
  typed(StyleValueKind.Number, finite(value, "number"));

export const bool = (
  value: boolean,
): TypedStyleValue<typeof StyleValueKind.Boolean> =>
  typed(StyleValueKind.Boolean, value ? 1 : 0);

/** Packed RGBA in `0xRRGGBBAA` order. */
export const rgba = (
  value: number,
): TypedStyleValue<typeof StyleValueKind.Color> =>
  typed(StyleValueKind.Color, finite(value, "rgba") >>> 0);

/** One Vello blurred-rounded-rectangle shadow layer. */
export interface Shadow {
  readonly offsetX: number;
  readonly offsetY: number;
  readonly spread: number;
  /** Gaussian standard deviation passed directly to Vello. */
  readonly stdDev: number;
  /** Packed sRGBA in `0xRRGGBBAA` order. */
  readonly color: number;
  /** Override the rounded-rectangle radius; defaults to the node radius plus spread. */
  readonly radius?: number;
}

export interface ShadowOptions {
  offsetX?: number;
  offsetY?: number;
  spread?: number;
  stdDev: number;
  color: number;
  radius?: number;
}

/** Construct and validate a Vello-native shadow layer. */
export function shadow(options: ShadowOptions): Shadow {
  const result: Shadow = {
    offsetX: options.offsetX ?? 0,
    offsetY: options.offsetY ?? 0,
    spread: options.spread ?? 0,
    stdDev: options.stdDev,
    color: options.color >>> 0,
    ...(options.radius === undefined ? {} : { radius: options.radius }),
  };
  for (const [name, value] of Object.entries(result)) {
    if (name === "color") continue;
    if (typeof value !== "number" || !Number.isFinite(value)) {
      throw new TypeError(`shadow ${name} must be a finite number`);
    }
  }
  if (result.stdDev < 0)
    throw new RangeError("shadow stdDev cannot be negative");
  if (result.radius !== undefined && result.radius < 0) {
    throw new RangeError("shadow radius cannot be negative");
  }
  return result;
}

export const auto = (): TypedStyleValue<typeof StyleValueKind.Auto> =>
  typed(StyleValueKind.Auto);

export function isTypedStyleValue(value: unknown): value is TypedStyleValue {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as Partial<TypedStyleValue>)[STYLE_VALUE] === true
  );
}

export function assertInlineStyleValue(property: string, value: unknown): void {
  const contract = (
    INLINE_STYLE_CONTRACT as Record<
      string,
      { string: boolean; number: boolean; typed: readonly number[] }
    >
  )[property];
  if (!contract) {
    const kebab = property.replace(
      /[A-Z]/g,
      (letter) => `-${letter.toLowerCase()}`,
    );
    const suggestion =
      kebab !== property && kebab in INLINE_STYLE_CONTRACT
        ? `; use ${kebab}`
        : "";
    throw new TypeError(
      `unsupported inline style property ${property}${suggestion}`,
    );
  }
  if (isTypedStyleValue(value)) {
    if (!Number.isFinite(value.value)) {
      throw new TypeError(`inline style ${property} must be finite`);
    }
    if (!contract.typed.includes(value.kind)) {
      throw new TypeError(
        `typed style kind ${value.kind} is invalid for ${property}`,
      );
    }
    return;
  }
  if (typeof value === "string" && contract.string) return;
  if (typeof value === "number" && contract.number) {
    if (!Number.isFinite(value)) {
      throw new TypeError(`inline style ${property} must be finite`);
    }
    return;
  }
  throw new TypeError(`invalid inline style value for ${property}`);
}

/** A 2D affine matrix in CSS/Vello `[a, b, c, d, e, f]` order. */
export type Affine2D = readonly [
  number,
  number,
  number,
  number,
  number,
  number,
];

/** Device-independent logical-pixel translation for runtime state. */
export const translate2d = (x: number, y: number): Affine2D => [
  1,
  0,
  0,
  1,
  x,
  y,
];

/** Rotation in radians; the native host pivots it around the border-box center. */
export const rotate2d = (angle: number): Affine2D => {
  const cosine = Math.cos(angle);
  const sine = Math.sin(angle);
  return [cosine, sine, -sine, cosine, 0, 0];
};

/** Type-check a list of utilities while producing Solid's class string. */
export const classes = (...values: readonly WabouUtility[]): string =>
  values.join(" ");
