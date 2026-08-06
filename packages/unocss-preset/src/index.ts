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
  dynamicRules: {
    resolver:
      | "spacing"
      | "dimension"
      | "color"
      | "opacity"
      | "number"
      | "ratio"
      | "length"
      | "translate"
      | "scale"
      | "rotate";
    prefixes: { name: string; properties: string[] }[];
  }[];
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

function matchDynamic(
  utility: string,
  resolver: Manifest["dynamicRules"][number]["resolver"],
):
  | { name: string; token: string; properties: string[]; negative: boolean }
  | undefined {
  const negative =
    (["spacing", "dimension", "translate", "rotate"] as string[]).includes(
      resolver,
    ) && utility.startsWith("-");
  const normalized = negative ? utility.slice(1) : utility;
  const rule = wabouUtilityManifest.dynamicRules.find(
    (candidate) => candidate.resolver === resolver,
  );
  for (const prefix of [...(rule?.prefixes ?? [])].sort(
    (left, right) => right.name.length - left.name.length,
  )) {
    const marker = `${prefix.name}-`;
    if (normalized.startsWith(marker)) {
      return {
        name: prefix.name,
        token: normalized.slice(marker.length),
        properties: prefix.properties,
        negative,
      };
    }
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

function negateLength(value: Length): Length | undefined {
  if (value.unit === "auto") return;
  return { ...value, value: -value.value };
}

const rustF32 = Math.fround;

const lengthDeclaration = (
  property: string,
  value: Length,
): WabouStyleDeclaration => ({
  property,
  value: { type: "length", value },
});

const transformDeclaration = (
  kind: string,
  value: WabouStyleValue,
): WabouStyleDeclaration => ({
  property:
    (
      {
        translateX: "transform-translate-x",
        translateY: "transform-translate-y",
        scale: "transform-scale",
        rotate: "transform-rotate",
      } as Record<string, string>
    )[kind] ?? "transform-component",
  value: {
    type: "list",
    values: [
      {
        type: "record",
        fields: {
          kind: { type: "keyword", value: kind },
          value,
        },
      },
    ],
  },
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
  const spacing = matchDynamic(matcher, "spacing");
  if (!declarations && spacing) {
    let value = parseLength(spacing.token, true);
    if (
      value?.unit === "auto" &&
      !spacing.properties.every((property) => property.startsWith("margin-"))
    )
      return {
        candidate,
        message: `invalid Wabou spacing in \`${candidate}\`; auto is only valid for margins`,
      };
    if (spacing.negative) {
      if (
        !spacing.properties.every((property) => property.startsWith("margin-"))
      )
        return {
          candidate,
          message: `invalid negative Wabou spacing in \`${candidate}\`; only margins may be negative`,
        };
      value = value && negateLength(value);
    }
    if (!value)
      return {
        candidate,
        message: `invalid Wabou spacing in \`${candidate}\`; expected a scale token, px, rem, or percentage`,
      };
    declarations = spacing.properties.map((property) =>
      lengthDeclaration(property, value),
    );
  }
  const dimension = matchDynamic(matcher, "dimension");
  if (!declarations && dimension) {
    let value =
      parseLength(dimension.token, false) ?? parseLength(dimension.token, true);
    if (dimension.negative) {
      if (
        !dimension.properties.every((property) =>
          ["top", "right", "bottom", "left"].includes(property),
        )
      )
        return {
          candidate,
          message: `invalid negative Wabou dimension in \`${candidate}\`; only positioned edges may be negative`,
        };
      value = value && negateLength(value);
    }
    if (!value)
      return {
        candidate,
        message: `invalid Wabou dimension in \`${candidate}\`; expected auto, full, a scale token, px, rem, or percentage`,
      };
    declarations = dimension.properties.map((name) =>
      lengthDeclaration(name, value),
    );
  }
  const lengthRule = matchDynamic(matcher, "length");
  if (!declarations && lengthRule) {
    const value = parseLength(lengthRule.token, false);
    if (value) {
      declarations = lengthRule.properties.map((property) =>
        lengthDeclaration(property, value),
      );
    }
  }
  const color = matchDynamic(matcher, "color");
  if (!declarations && color) {
    const [colorName, opacityToken] = color.token.split("/", 2);
    let rgba: number | undefined = wabouUtilityManifest.colors[colorName];
    const arbitrary = colorName.match(
      /^\[#([0-9a-fA-F]{6}|[0-9a-fA-F]{8})\]$/,
    )?.[1];
    if (rgba === undefined && arbitrary) {
      rgba = Number.parseInt(arbitrary, 16);
      if (arbitrary.length === 6) rgba = ((rgba << 8) | 0xff) >>> 0;
    }
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
        property: color.properties[0],
        value: { type: "color", value: { kind: "literal", rgba } },
      },
    ];
  }
  const opacityRule = matchDynamic(matcher, "opacity");
  if (!declarations && opacityRule) {
    const opacity = Number(opacityRule.token);
    if (!Number.isFinite(opacity) || opacity < 0 || opacity > 100)
      return {
        candidate,
        message: `invalid Wabou opacity in \`${candidate}\``,
      };
    declarations = [
      {
        property: opacityRule.properties[0],
        value: { type: "number", value: opacity / 100 },
      },
    ];
  }
  const numberRule = matchDynamic(matcher, "number");
  if (!declarations && numberRule) {
    const raw = numberRule.token.match(
      /^\[(-?(?:\d+(?:\.\d*)?|\.\d+))\]$/,
    )?.[1];
    const value = raw === undefined ? Number.NaN : Number(raw);
    if (!Number.isFinite(value))
      return {
        candidate,
        message: `invalid Wabou number in \`${candidate}\`; expected an arbitrary finite number`,
      };
    declarations = [
      { property: numberRule.properties[0], value: { type: "number", value } },
    ];
  }
  const ratioRule = matchDynamic(matcher, "ratio");
  if (!declarations && ratioRule) {
    const raw =
      ratioRule.token.startsWith("[") && ratioRule.token.endsWith("]")
        ? ratioRule.token.slice(1, -1)
        : undefined;
    const parts = raw?.split("/", 2);
    const value = rustF32(
      parts?.length === 2 ? Number(parts[0]) / Number(parts[1]) : Number(raw),
    );
    if (!Number.isFinite(value) || value <= 0)
      return {
        candidate,
        message: `invalid Wabou ratio in \`${candidate}\`; expected an arbitrary positive ratio`,
      };
    declarations = [
      { property: ratioRule.properties[0], value: { type: "number", value } },
    ];
  }
  const translateRule = matchDynamic(matcher, "translate");
  if (!declarations && translateRule) {
    let value = parseLength(translateRule.token, true);
    if (translateRule.negative) value = value && negateLength(value);
    if (!value)
      return {
        candidate,
        message: `invalid Wabou translate in \`${candidate}\``,
      };
    declarations = [
      {
        property: translateRule.properties[0],
        value: {
          type: "list",
          values: [
            {
              type: "record",
              fields: {
                kind: {
                  type: "keyword",
                  value:
                    translateRule.name === "translate-x"
                      ? "translateX"
                      : "translateY",
                },
                value: { type: "length", value },
              },
            },
          ],
        },
      },
    ];
  }
  const scaleRule = matchDynamic(matcher, "scale");
  if (!declarations && scaleRule) {
    const arbitrary = scaleRule.token.match(
      /^\[(-?(?:\d+(?:\.\d*)?|\.\d+))\]$/,
    )?.[1];
    const scale = rustF32(
      arbitrary === undefined
        ? Number(scaleRule.token) / 100
        : Number(arbitrary),
    );
    if (!Number.isFinite(scale))
      return { candidate, message: `invalid Wabou scale in \`${candidate}\`` };
    declarations = [
      transformDeclaration("scale", {
        type: "list",
        values: [
          { type: "number", value: scale },
          { type: "number", value: scale },
        ],
      }),
    ];
  }
  const rotateRule = matchDynamic(matcher, "rotate");
  if (!declarations && rotateRule) {
    const arbitrary = rotateRule.token.match(
      /^\[(-?(?:\d+(?:\.\d*)?|\.\d+))\]$/,
    )?.[1];
    let degrees = Number(arbitrary ?? rotateRule.token);
    if (rotateRule.negative) degrees = -degrees;
    const radians = rustF32((degrees * Math.PI) / 180);
    if (!Number.isFinite(radians))
      return {
        candidate,
        message: `invalid Wabou rotation in \`${candidate}\``,
      };
    declarations = [
      transformDeclaration("rotate", { type: "number", value: radians }),
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
          if (kind?.type !== "keyword") return "";
          if (kind.value === "repeat") {
            const count = item.fields.count;
            const tracks = item.fields.values;
            if (count?.type !== "number" || tracks?.type !== "list") return "";
            return `repeat(${count.value}, ${tracks.values.map(cssValue).join(" ")})`;
          }
          if (!argument) return "";
          if (kind.value === "breadth") return cssValue(argument);
          if (kind.value === "flex") return `${cssValue(argument)}fr`;
          const text =
            argument.type === "list"
              ? argument.values.map(cssValue).join(", ")
              : cssValue(argument);
          return `${kind.value}(${text})`;
        })
        .join(" ");
    case "record": {
      const kind = value.fields.kind;
      const argument = value.fields.value;
      if (kind?.type !== "keyword" || !argument) return "";
      if (kind.value === "breadth") return cssValue(argument);
      if (kind.value === "flex") return `${cssValue(argument)}fr`;
      return "";
    }
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
          property.startsWith("transform-") ? "transform" : property,
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
