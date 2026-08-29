export {
  INLINE_STYLE_CONTRACT,
  type WabouStyle,
} from "./generated/style-properties.ts";
export * from "./generated/utility-types.ts";

import { INLINE_STYLE_CONTRACT } from "./generated/style-properties.ts";
import type { WabouUtility } from "./generated/utility-types.ts";
import { UTILITY_CONFLICT_DATA } from "./generated/utility-conflicts.ts";

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

/** Uniform scaling; the native host pivots it around the border-box center. */
export const scale2d = (scale: number): Affine2D => [scale, 0, 0, scale, 0, 0];

/** Rotation in radians; the native host pivots it around the border-box center. */
export const rotate2d = (angle: number): Affine2D => {
  const cosine = Math.cos(angle);
  const sine = Math.sin(angle);
  return [cosine, sine, -sine, cosine, 0, 0];
};

/** Type-check a list of utilities while producing Solid's class string. */
export const classes = (...values: readonly WabouUtility[]): string =>
  values.join(" ");

export type ClassValue = string | false | null | undefined;

const staticConflictProperties = new Map<string, readonly string[]>(
  Object.entries(UTILITY_CONFLICT_DATA.staticUtilities),
);
const spacingTokens = new Set<string>(UTILITY_CONFLICT_DATA.spacing);
const colorTokens = new Set<string>(UTILITY_CONFLICT_DATA.colors);
const dynamicConflictRules = [...UTILITY_CONFLICT_DATA.dynamicRules].sort(
  (left, right) => right.name.length - left.name.length,
);
const dynamicConflictCache = new Map<string, readonly string[] | null>();
const arbitraryNumber = /^\[-?(?:\d+(?:\.\d*)?|\.\d+)\]$/;
const arbitraryLength =
  /^\[-?(?:\d+(?:\.\d*)?|\.\d+)(?:px|rem|%)\]$/;
const barePercent = /^-?(?:\d+(?:\.\d*)?|\.\d+)%$/;
const fraction = /^\d+\/[1-9]\d*$/;

function dynamicTokenMatches(
  resolver: string,
  token: string,
  allowAuto: boolean,
): boolean {
  const colorAndOpacity = token.split("/", 2);
  switch (resolver) {
    case "spacing":
      return (
        spacingTokens.has(token) ||
        (allowAuto && token === "auto") ||
        arbitraryLength.test(token)
      );
    case "dimension":
      return (
        spacingTokens.has(token) ||
        token === "full" ||
        arbitraryLength.test(token) ||
        barePercent.test(token) ||
        fraction.test(token)
      );
    case "color":
      return (
        ((colorTokens.has(colorAndOpacity[0] ?? "") ||
          /^[a-z][a-z0-9-]*$/.test(colorAndOpacity[0] ?? "")) &&
          (colorAndOpacity.length === 1 ||
            /^(?:100|\d{1,2})$/.test(colorAndOpacity[1] ?? ""))) ||
        /^\[#[0-9a-fA-F]{3,8}\](?:\/\d+)?$/.test(token)
      );
    case "opacity":
      return (
        /^\d+(?:\.\d+)?$/.test(token) &&
        Number(token) >= 0 &&
        Number(token) <= 100
      );
    case "number":
    case "scale":
    case "rotate":
      return arbitraryNumber.test(token) || /^-?(?:\d+(?:\.\d*)?|\.\d+)$/.test(token);
    case "ratio":
      return /^\[(?:\d+(?:\.\d*)?|\.\d+)(?:\/(?:\d+(?:\.\d*)?|\.\d+))?\]$/.test(
        token,
      );
    case "length":
      return arbitraryLength.test(token);
    case "translate":
      return spacingTokens.has(token) || arbitraryLength.test(token);
    default:
      return false;
  }
}

/** Return the Style IR properties affected by a supported utility candidate. */
export function utilityConflictProperties(
  candidate: string,
): readonly string[] | undefined {
  const staticProperties = staticConflictProperties.get(candidate);
  if (staticProperties) return staticProperties;
  const cached = dynamicConflictCache.get(candidate);
  if (cached !== undefined) return cached ?? undefined;
  const negative = candidate.startsWith("-");
  const normalized = negative ? candidate.slice(1) : candidate;
  for (const rule of dynamicConflictRules) {
    const marker = `${rule.name}-`;
    if (!normalized.startsWith(marker)) continue;
    if (negative && !rule.negative) continue;
    const token = normalized.slice(marker.length);
    if (dynamicTokenMatches(rule.resolver, token, rule.auto)) {
      dynamicConflictCache.set(candidate, rule.properties);
      return rule.properties;
    }
  }
  dynamicConflictCache.set(candidate, null);
}

/**
 * Merge Wabou utility strings using their Style IR properties. Later values
 * win, while utilities that still contribute an uncovered property remain.
 * Unknown third-party classes are preserved unchanged.
 */
export function mergeClasses(...values: readonly ClassValue[]): string {
  const candidates = values.flatMap((value) =>
    typeof value === "string" ? value.trim().split(/\s+/).filter(Boolean) : [],
  );
  const covered = new Set<string>();
  const merged: string[] = [];
  for (let index = candidates.length - 1; index >= 0; index -= 1) {
    const candidate = candidates[index];
    if (!candidate) continue;
    const properties = utilityConflictProperties(candidate);
    if (!properties) {
      merged.push(candidate);
      continue;
    }
    if (properties.every((property) => covered.has(property))) continue;
    merged.push(candidate);
    for (const property of properties) covered.add(property);
  }
  return merged.reverse().join(" ");
}
