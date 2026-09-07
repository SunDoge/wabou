const ORB_FLOAT_COUNT = 136;
const COLOR_OFFSET = 40;

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

/** Uniform snapshot for Liquid Orb Editor's single-pass blue-drop preset. */
export function createBlueDropUniforms(): readonly number[] {
  const values = new Float32Array(ORB_FLOAT_COUNT);
  values.set([
    0, 0, 0, 1, 0.74, 0.48, 2.65, 0.42, 2.4, 0.16, 0.22, 0.42, 0.32, 0.24, 1.24,
    20, 0.005, 0, 0, 1, 0.66, 0.08, 2, 0.42, 0.77, 0.23, 65, 0, 0, 1, 0.22,
    0.25, 0.72, 5, 0.42, 1.25, 0.55, 0.3, 1.2, 0.7,
  ]);

  const colors = [
    "#020B1D",
    "#0756B8",
    "#1EC8FF",
    "#DDFBFF",
    "#EAFBFF",
    "#F6FDFF",
    "#4FD7FF",
    "#466DFF",
    "#DDFBFF",
    "#A8D9FF",
    "#010207",
    "#168DFF",
    ...PALETTE_STOPS,
  ];
  colors.forEach((color, index) => {
    values.set(rgba(color), COLOR_OFFSET + index * 4);
  });
  return Array.from(values);
}
