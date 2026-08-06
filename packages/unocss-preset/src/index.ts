import type { Preset, Rule } from "@unocss/core";
import manifestJson from "../generated/manifest.json" with { type: "json" };

type Length = { unit: "px" | "percent"; value: number } | { unit: "auto" };
export type WabouStyleValue =
  | { type: "keyword"; value: string }
  | { type: "boolean"; value: boolean }
  | { type: "number"; value: number }
  | { type: "length"; value: Length }
  | { type: "color"; value: { kind: "literal"; rgba: number } }
  | { type: "list"; values: WabouStyleValue[] }
  | { type: "record"; fields: Record<string, WabouStyleValue> };
export type WabouStyleDeclaration = {
  property: string;
  value: WabouStyleValue;
};
type RustParsedUtility = {
  className: string;
  declarations: WabouStyleDeclaration[];
};
type Manifest = {
  version: number;
  spacing: Record<string, number>;
  colors: Record<string, number>;
  staticUtilities: Record<string, WabouStyleDeclaration[]>;
  conformance: RustParsedUtility[];
};

export const wabouUtilityManifest = manifestJson as Manifest;

export type UtilityDiagnostic = {
  candidate: string;
  message: string;
};

export type ResolvedUtility = {
  candidate: string;
  matcher: string;
  declarations: WabouStyleDeclaration[];
};

const spacingPrefixes = [
  "gap",
  "px",
  "py",
  "pt",
  "pr",
  "pb",
  "pl",
  "mx",
  "my",
  "mt",
  "mr",
  "mb",
  "ml",
  "p",
  "m",
] as const;
const dimensionPrefixes = [
  "min-w",
  "min-h",
  "max-w",
  "max-h",
  "inset",
  "right",
  "bottom",
  "left",
  "top",
  "w",
  "h",
] as const;

const edges: Record<string, string[]> = {
  p: ["padding-top", "padding-right", "padding-bottom", "padding-left"],
  px: ["padding-left", "padding-right"],
  py: ["padding-top", "padding-bottom"],
  pt: ["padding-top"],
  pr: ["padding-right"],
  pb: ["padding-bottom"],
  pl: ["padding-left"],
  m: ["margin-top", "margin-right", "margin-bottom", "margin-left"],
  mx: ["margin-left", "margin-right"],
  my: ["margin-top", "margin-bottom"],
  mt: ["margin-top"],
  mr: ["margin-right"],
  mb: ["margin-bottom"],
  ml: ["margin-left"],
  gap: ["row-gap", "column-gap"],
};

function splitPrefix(
  utility: string,
  prefixes: readonly string[],
): [string, string] | undefined {
  for (const prefix of prefixes) {
    const marker = `${prefix}-`;
    if (utility.startsWith(marker))
      return [prefix, utility.slice(marker.length)];
  }
}

function parseLength(token: string, spacing: boolean): Length | undefined {
  if (token === "auto") return { unit: "auto" };
  if (token === "full") return { unit: "percent", value: 1 };
  if (spacing && token === "px") return { unit: "px", value: 1 };
  const scale = spacing ? wabouUtilityManifest.spacing[token] : undefined;
  if (scale !== undefined) return { unit: "px", value: scale };
  if (token.endsWith("%")) {
    const value = Number(token.slice(0, -1));
    if (Number.isFinite(value)) return { unit: "percent", value: value / 100 };
  }
  const raw =
    token.startsWith("[") && token.endsWith("]")
      ? token.slice(1, -1)
      : undefined;
  if (!raw) return;
  if (raw.endsWith("px")) {
    const value = Number(raw.slice(0, -2));
    if (Number.isFinite(value)) return { unit: "px", value };
  }
  if (raw.endsWith("rem")) {
    const value = Number(raw.slice(0, -3));
    if (Number.isFinite(value)) return { unit: "px", value: value * 16 };
  }
  if (raw.endsWith("%")) {
    const value = Number(raw.slice(0, -1));
    if (Number.isFinite(value)) return { unit: "percent", value: value / 100 };
  }
}

const lengthDeclaration = (
  property: string,
  value: Length,
): WabouStyleDeclaration => ({
  property,
  value: { type: "length", value },
});

function parseCandidate(
  candidate: string,
): ResolvedUtility | UtilityDiagnostic {
  const matcher = candidate;
  if (matcher.includes(":"))
    return {
      candidate,
      message: `Wabou variants are not supported in \`${candidate}\`; use Solid classList or typed style`,
    };

  let declarations = wabouUtilityManifest.staticUtilities[matcher];
  const spacing = splitPrefix(matcher, spacingPrefixes);
  if (!declarations && spacing) {
    const value = parseLength(spacing[1], true);
    if (!value)
      return {
        candidate,
        message: `invalid Wabou spacing in \`${candidate}\`; expected a scale token, px, rem, or percentage`,
      };
    declarations = edges[spacing[0]].map((property) =>
      lengthDeclaration(property, value),
    );
  }
  const dimension = splitPrefix(matcher, dimensionPrefixes);
  if (!declarations && dimension) {
    const value =
      parseLength(dimension[1], false) ?? parseLength(dimension[1], true);
    if (!value)
      return {
        candidate,
        message: `invalid Wabou dimension in \`${candidate}\`; expected auto, full, a scale token, px, rem, or percentage`,
      };
    const property =
      (
        {
          w: "width",
          h: "height",
          "min-w": "min-width",
          "min-h": "min-height",
          "max-w": "max-width",
          "max-h": "max-height",
        } as Record<string, string>
      )[dimension[0]] ?? dimension[0];
    declarations = (
      dimension[0] === "inset" ? ["top", "right", "bottom", "left"] : [property]
    ).map((name) => lengthDeclaration(name, value));
  }
  const color = splitPrefix(matcher, ["border", "text", "bg"]);
  if (!declarations && color) {
    const [colorName, opacityToken] = color[1].split("/", 2);
    let rgba: number | undefined = wabouUtilityManifest.colors[colorName];
    if (rgba !== undefined && opacityToken !== undefined) {
      const opacity = Number(opacityToken);
      rgba =
        Number.isFinite(opacity) && opacity >= 0 && opacity <= 100
          ? ((rgba & 0xffffff00) | Math.round(opacity * 2.55)) >>> 0
          : undefined;
    }
    if (rgba === undefined)
      return {
        candidate,
        message: `unknown Wabou theme color in \`${candidate}\``,
      };
    declarations = [
      {
        property:
          color[0] === "bg"
            ? "background-color"
            : color[0] === "text"
              ? "color"
              : "border-color",
        value: { type: "color", value: { kind: "literal", rgba } },
      },
    ];
  }
  if (!declarations && matcher.startsWith("opacity-")) {
    const opacity = Number(matcher.slice(8));
    if (!Number.isFinite(opacity) || opacity < 0 || opacity > 100)
      return {
        candidate,
        message: `invalid Wabou opacity in \`${candidate}\``,
      };
    declarations = [
      { property: "opacity", value: { type: "number", value: opacity / 100 } },
    ];
  }
  if (!declarations)
    return { candidate, message: `unsupported Wabou utility \`${candidate}\`` };
  return { candidate, matcher, declarations };
}

export function resolveWabouUtility(
  candidate: string,
): ResolvedUtility | undefined {
  const result = parseCandidate(candidate);
  return "declarations" in result ? result : undefined;
}

export function validateWabouUtility(
  candidate: string,
): UtilityDiagnostic | undefined {
  const result = parseCandidate(candidate);
  return "message" in result ? result : undefined;
}

function cssValue(value: WabouStyleValue): string | number {
  switch (value.type) {
    case "keyword":
      return value.value;
    case "boolean":
      return String(value.value);
    case "number":
      return value.value;
    case "length":
      if (value.value.unit === "auto") return "auto";
      return `${value.value.unit === "percent" ? value.value.value * 100 : value.value.value}${value.value.unit === "percent" ? "%" : "px"}`;
    case "color":
      return `#${value.value.rgba.toString(16).padStart(8, "0")}`;
    case "list":
      return value.values
        .map((item) => {
          if (item.type !== "record") return cssValue(item);
          const kind = item.fields.kind;
          const argument = item.fields.value;
          if (kind?.type !== "keyword" || !argument) return "";
          const text =
            argument.type === "list"
              ? argument.values.map(cssValue).join(", ")
              : cssValue(argument);
          return `${kind.value}(${text})`;
        })
        .join(" ");
    case "record":
      return "";
  }
}

function unoRule(): Rule {
  return [
    /^.+$/,
    ([candidate]) => {
      const resolved = resolveWabouUtility(candidate);
      if (!resolved) return;
      return Object.fromEntries(
        resolved.declarations.map(({ property, value }) => [
          property,
          cssValue(value),
        ]),
      );
    },
  ];
}

/** UnoCSS adapter for editor tooling over the native utility manifest. */
export function presetWabou(): Preset {
  return {
    name: "@wabou/unocss-preset",
    rules: [unoRule()],
    autocomplete: {
      templates: [
        "p-$spacing",
        "px-$spacing",
        "py-$spacing",
        "m-$spacing",
        "gap-$spacing",
        "w-$spacing",
        "h-$spacing",
        "bg-$colors",
        "text-$colors",
        "border-$colors",
      ],
    },
  };
}
