/**
 * CSS support matrix — compiler ↔ host contract.
 *
 * - **supported**: compile to Style IR and apply in Rust `apply_ir`.
 * - **unsupported**: compile-time error (never emit IR).
 *
 * Source of truth: `./css-support-matrix.json` (also `include_str!`'d by
 * wabou-runtime tests so Rust cannot drift).
 */

import matrixJson from "./css-support-matrix.json" with { type: "json" };

export type SupportKind = "supported" | "unsupported";

type SampleSpec = string | { sample: string; rustOnly?: boolean };

export type CssSupportMatrix = {
  version: number;
  description: string;
  supported: Record<string, SampleSpec>;
  unsupported: Record<string, string>;
  unsupportedPrefixes: Record<string, string>;
};

export const CSS_SUPPORT_MATRIX = matrixJson as CssSupportMatrix;

function sampleOf(spec: SampleSpec): string {
  return typeof spec === "string" ? spec : spec.sample;
}

function isRustOnly(spec: SampleSpec): boolean {
  return typeof spec === "object" && !!spec.rustOnly;
}

/** Every property name Rust `apply_ir` must accept. */
export function allHostProperties(): string[] {
  return Object.keys(CSS_SUPPORT_MATRIX.supported).sort();
}

/** Properties the compiler is allowed to emit (excludes rust-only aliases). */
export function allCompilerProperties(): string[] {
  const out: string[] = [];
  for (const [name, spec] of Object.entries(CSS_SUPPORT_MATRIX.supported)) {
    if (!isRustOnly(spec)) out.push(name);
  }
  return out.sort();
}

export function propertySample(property: string): string | undefined {
  const supported = CSS_SUPPORT_MATRIX.supported[property];
  if (supported) return sampleOf(supported);
  return undefined;
}

/**
 * Classify a property for the compiler. Returns a human-readable reject
 * message when the property must not enter Style IR.
 */
export function rejectUnsupportedProperty(
  property: string,
): string | undefined {
  if (property in CSS_SUPPORT_MATRIX.supported) return;
  if (property in CSS_SUPPORT_MATRIX.unsupported) {
    return `unsupported CSS property ${property}: ${CSS_SUPPORT_MATRIX.unsupported[property]}`;
  }
  for (const [prefix, reason] of Object.entries(
    CSS_SUPPORT_MATRIX.unsupportedPrefixes,
  )) {
    if (property.startsWith(prefix)) {
      return `unsupported CSS property ${property}: ${reason}`;
    }
  }
  // Not on the matrix at all — refuse rather than silent IR that Rust ignores.
  return `unsupported CSS property ${property}: not in the wabou CSS support matrix (add it as supported or unsupported)`;
}

export function supportKind(property: string): SupportKind | "unknown" {
  if (property in CSS_SUPPORT_MATRIX.supported) return "supported";
  if (property in CSS_SUPPORT_MATRIX.unsupported) return "unsupported";
  for (const prefix of Object.keys(CSS_SUPPORT_MATRIX.unsupportedPrefixes)) {
    if (property.startsWith(prefix)) return "unsupported";
  }
  return "unknown";
}
