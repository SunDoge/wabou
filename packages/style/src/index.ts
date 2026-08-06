export * from "../generated/utility-types.ts";

import type { WabouUtility } from "../generated/utility-types.ts";

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

export interface TypedStyleValue {
  readonly [STYLE_VALUE]: true;
  readonly kind: StyleValueKind;
  readonly value: number;
}

function typed(kind: StyleValueKind, value = 0): TypedStyleValue {
  return { [STYLE_VALUE]: true, kind, value };
}

/** Device-independent logical pixels. */
export const px = (value: number): TypedStyleValue =>
  typed(StyleValueKind.Px, value);

/** A ratio where `1` means 100%. */
export const percent = (value: number): TypedStyleValue =>
  typed(StyleValueKind.Percent, value);

/** A unitless numeric style value such as opacity or flex-grow. */
export const number = (value: number): TypedStyleValue =>
  typed(StyleValueKind.Number, value);

export const bool = (value: boolean): TypedStyleValue =>
  typed(StyleValueKind.Boolean, value ? 1 : 0);

/** Packed RGBA in `0xRRGGBBAA` order. */
export const rgba = (value: number): TypedStyleValue =>
  typed(StyleValueKind.Color, value >>> 0);

export const auto = (): TypedStyleValue => typed(StyleValueKind.Auto);

export function isTypedStyleValue(value: unknown): value is TypedStyleValue {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as Partial<TypedStyleValue>)[STYLE_VALUE] === true
  );
}

export type WabouStyle = Record<
  string,
  string | number | TypedStyleValue | null | undefined
>;

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

/** Type-check a list of utilities while producing Solid's class string. */
export const classes = (...values: readonly WabouUtility[]): string =>
  values.join(" ");
