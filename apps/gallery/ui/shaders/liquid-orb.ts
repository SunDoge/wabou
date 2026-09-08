const ORB_FLOAT_COUNT = 136;
const COLOR_OFFSET = 40;

export type LiquidOrbPreset = "siri" | "blueDrop" | "refractiveBlob";

const SCALAR_KEYS = [
  "speed",
  "radius",
  "zoom",
  "warp",
  "ridgeAmt",
  "sharp",
  "shade",
  "sheen",
  "gloss",
  "shellMidAlpha",
  "shellEdgeAlpha",
  "exposure",
  "style",
  "edgeSoftness",
  "edgeGlow",
  "paletteCount",
  "glassEnabled",
  "glassOpacity",
  "contourDeform",
  "bandDensity",
  "chromaticShift",
  "metalScale",
  "metalStretch",
  "metalAngle",
  "metalOffset",
  "metalPhase",
  "metalEvolution",
  "metalRoughness",
  "metalDepth",
  "particleDensity",
  "ribbonCount",
  "ribbonWidth",
  "ribbonTwist",
  "ribbonFold",
  "ribbonBreath",
  "particleSize",
  "particleBloom",
] as const;

const COLOR_KEYS = [
  "colorA",
  "colorB",
  "colorC",
  "colorD",
  "highlightColor",
  "shellInner",
  "shellMid",
  "shellEdge",
  "sheenColor",
  "specColor",
  "canvasColor",
  "glowColor",
] as const;

type ScalarKey = (typeof SCALAR_KEYS)[number];
type ColorKey = (typeof COLOR_KEYS)[number];
type OrbPresetValues = {
  scalars: Partial<Record<ScalarKey, number>>;
  colors: Partial<Record<ColorKey, string>>;
};

const BASE_SCALARS: Record<ScalarKey, number> = {
  speed: 1,
  radius: 0.72,
  zoom: 0.3,
  warp: 3,
  ridgeAmt: 0.5,
  sharp: 2.2,
  shade: 0.3,
  sheen: 0.36,
  gloss: 0.28,
  shellMidAlpha: 0.2,
  shellEdgeAlpha: 0.22,
  exposure: 1,
  style: 9,
  edgeSoftness: 0.005,
  edgeGlow: 0,
  paletteCount: 0,
  glassEnabled: 1,
  glassOpacity: 0.42,
  contourDeform: 0,
  bandDensity: 2,
  chromaticShift: 0.42,
  metalScale: 0.77,
  metalStretch: 0.23,
  metalAngle: 65,
  metalOffset: 0,
  metalPhase: 0,
  metalEvolution: 1,
  metalRoughness: 0.22,
  metalDepth: 0.25,
  particleDensity: 0.72,
  ribbonCount: 5,
  ribbonWidth: 0.42,
  ribbonTwist: 1.25,
  ribbonFold: 0.55,
  ribbonBreath: 0.3,
  particleSize: 1.2,
  particleBloom: 0.7,
};

const BASE_COLORS: Record<ColorKey, string> = {
  colorA: "#F7FBFF",
  colorB: "#D6E8F7",
  colorC: "#A8C8F0",
  colorD: "#6F9EE8",
  highlightColor: "#FFFFFF",
  shellInner: "#FFFFFF",
  shellMid: "#D6E8F7",
  shellEdge: "#6F9EE8",
  sheenColor: "#EAF4FF",
  specColor: "#DCEAFF",
  canvasColor: "#000000",
  glowColor: "#6F9EE8",
};

const PRESETS: Record<LiquidOrbPreset, OrbPresetValues> = {
  siri: {
    scalars: {
      speed: 0.82,
      zoom: 0.36,
      warp: 3.2,
      shade: 0.12,
      sheen: 0.28,
      gloss: 0.24,
      glassOpacity: 0.44,
      shellMidAlpha: 0.18,
      shellEdgeAlpha: 0.18,
      exposure: 2,
      style: 9,
    },
    colors: {
      colorA: "#FFD86B",
      colorB: "#82F4FF",
      colorC: "#FF7BD5",
      colorD: "#8E6CFF",
      shellMid: "#9BF4FF",
      shellEdge: "#C5A9FF",
      canvasColor: "#030409",
      glowColor: "#956CFF",
    },
  },
  blueDrop: {
    scalars: {
      speed: 0.9,
      radius: 0.74,
      contourDeform: 0.08,
      zoom: 0.48,
      warp: 2.65,
      ridgeAmt: 0.42,
      sharp: 2.4,
      shade: 0.16,
      sheen: 0.22,
      gloss: 0.42,
      glassOpacity: 0.66,
      shellMidAlpha: 0.32,
      shellEdgeAlpha: 0.24,
      exposure: 1.24,
      style: 20,
    },
    colors: {
      colorA: "#020B1D",
      colorB: "#0756B8",
      colorC: "#1EC8FF",
      colorD: "#DDFBFF",
      highlightColor: "#EAFBFF",
      shellInner: "#F6FDFF",
      shellMid: "#4FD7FF",
      shellEdge: "#466DFF",
      sheenColor: "#DDFBFF",
      specColor: "#A8D9FF",
      canvasColor: "#010207",
      glowColor: "#168DFF",
    },
  },
  refractiveBlob: {
    scalars: {
      speed: 0.76,
      radius: 0.73,
      contourDeform: 0.16,
      zoom: 0.46,
      warp: 3.65,
      ridgeAmt: 0.58,
      sharp: 2.7,
      shade: 0.14,
      sheen: 0.14,
      gloss: 0.52,
      glassOpacity: 0.82,
      shellMidAlpha: 0.42,
      shellEdgeAlpha: 0.2,
      exposure: 1.2,
      style: 23,
    },
    colors: {
      colorA: "#1B102B",
      colorB: "#7056A8",
      colorC: "#BFA5F5",
      colorD: "#F1E8FF",
      shellInner: "#F6F0FF",
      shellMid: "#D9C7FF",
      shellEdge: "#B59AE8",
      sheenColor: "#FFFFFF",
      specColor: "#E9DEFF",
      canvasColor: "#050208",
      glowColor: "#B18CFF",
    },
  },
};

const PALETTE_STOPS = [
  "#F7FBFF",
  "#EFF6FD",
  "#E0EEF9",
  "#D4E6F7",
  "#BBD5F3",
  "#A6C7F0",
  "#87B0EB",
  "#6F9EE8",
  "#6F9EE8",
  "#6F9EE8",
  "#6F9EE8",
  "#6F9EE8",
] as const;

function rgba(hex: string): readonly number[] {
  const value = hex.slice(1);
  return [
    Number.parseInt(value.slice(0, 2), 16) / 255,
    Number.parseInt(value.slice(2, 4), 16) / 255,
    Number.parseInt(value.slice(4, 6), 16) / 255,
    1,
  ];
}

/** Build one uniform snapshot for Liquid Orb Editor's single-pass style bank. */
export function createLiquidOrbUniforms(
  preset: LiquidOrbPreset,
): readonly number[] {
  const values = new Float32Array(ORB_FLOAT_COUNT);
  const scalars = { ...BASE_SCALARS, ...PRESETS[preset].scalars };
  const colors = { ...BASE_COLORS, ...PRESETS[preset].colors };
  SCALAR_KEYS.forEach((key, index) => {
    values[index + 3] = scalars[key];
  });
  [...COLOR_KEYS.map((key) => colors[key]), ...PALETTE_STOPS].forEach(
    (color, index) => values.set(rgba(color), COLOR_OFFSET + index * 4),
  );
  return Array.from(values);
}
