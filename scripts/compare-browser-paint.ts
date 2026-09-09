import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const output = resolve(process.argv[2] ?? "target/render-reference");
const wabou =
  process.env.WABOU_RENDER_COMMAND ?? resolve(root, "target/release/wabou");
const chrome =
  process.env.CHROME ??
  ["google-chrome", "chromium", "chromium-browser"].find((candidate) =>
    Bun.which(candidate),
  );
const magick = process.env.MAGICK ?? Bun.which("magick");

if (!chrome) {
  throw new Error(
    "Chrome/Chromium is required (or set CHROME=/path/to/browser)",
  );
}
if (!magick) {
  throw new Error("ImageMagick is required (or set MAGICK=/path/to/magick)");
}
if (!(await Bun.file(wabou).exists())) {
  throw new Error(
    `release Wabou CLI not found at ${wabou}; run cargo build --release -p wabou-cli`,
  );
}

await mkdir(output, { recursive: true });
const nativePng = resolve(output, "wabou.png");
const nativeSnapshot = resolve(output, "wabou.json");
const browserPng = resolve(output, "chromium.png");
const differencePng = resolve(output, "difference.png");
const comparisonPng = resolve(output, "comparison.png");
const referenceHtml = resolve(
  root,
  "tests/render-reference/paint-reference.html",
);

async function run(command: string, args: string[], allowDifference = false) {
  const child = Bun.spawn([command, ...args], {
    cwd: root,
    stdout: "inherit",
    stderr: allowDifference ? "pipe" : "inherit",
  });
  const stderr =
    child.stderr === null ? "" : await new Response(child.stderr).text();
  const status = await child.exited;
  if (status !== 0 && !(allowDifference && status === 1)) {
    throw new Error(`${command} exited with status ${status}`);
  }
  return stderr.trim();
}

await run(wabou, [
  "render",
  "apps/gallery",
  "--fixture",
  "foundations/paint-reference",
  "--wait-ms",
  "0",
  "--out",
  nativePng,
  "--snapshot",
  nativeSnapshot,
]);

await run(chrome, [
  "--headless=new",
  "--no-sandbox",
  "--disable-dev-shm-usage",
  "--hide-scrollbars",
  "--force-device-scale-factor=1",
  "--window-size=640,420",
  `--screenshot=${browserPng}`,
  `file://${referenceHtml}`,
]);

const rmseOutput = await run(
  magick,
  ["compare", "-metric", "RMSE", nativePng, browserPng, differencePng],
  true,
);
const match = rmseOutput.match(/\((?<normalized>[0-9.]+)\)/);
const normalizedRmse = Number(match?.groups?.normalized ?? Number.NaN);

const regions = {
  solid: "112x80+40+40",
  alpha: "120x84+192+40",
  roundedClip: "148x100+38+158",
  shadow: "190x130+209+143",
  translucent: "384x64+40+288",
} as const;
const regionRmse: Record<keyof typeof regions, number> = {} as Record<
  keyof typeof regions,
  number
>;
for (const [name, geometry] of Object.entries(regions) as [
  keyof typeof regions,
  string,
][]) {
  const nativeCrop = resolve(output, `${name}-wabou.png`);
  const browserCrop = resolve(output, `${name}-chromium.png`);
  const differenceCrop = resolve(output, `${name}-difference.png`);
  await run(magick, [nativePng, "-crop", geometry, "+repage", nativeCrop]);
  await run(magick, [browserPng, "-crop", geometry, "+repage", browserCrop]);
  const metric = await run(
    magick,
    ["compare", "-metric", "RMSE", nativeCrop, browserCrop, differenceCrop],
    true,
  );
  regionRmse[name] = Number(
    metric.match(/\((?<normalized>[0-9.]+)\)/)?.groups?.normalized ??
      Number.NaN,
  );
}

await run(magick, [
  nativePng,
  browserPng,
  differencePng,
  "+append",
  comparisonPng,
]);

const report = {
  version: 1,
  viewport: { width: 640, height: 420, scaleFactor: 1 },
  normalizedRmse,
  regionRmse,
  note: "Chromium is a comparison oracle for solid color, alpha, clipping, and approximate shadow profiles; glyphs and backend-specific antialiasing require separate tolerances.",
  outputs: {
    wabou: nativePng,
    chromium: browserPng,
    difference: differencePng,
    comparison: comparisonPng,
    snapshot: nativeSnapshot,
  },
};
await Bun.write(
  resolve(output, "report.json"),
  `${JSON.stringify(report, null, 2)}\n`,
);
console.log(
  `[wabou] browser paint comparison RMSE=${normalizedRmse.toFixed(6)}`,
);
console.log(
  `[wabou] regions ${Object.entries(regionRmse)
    .map(([name, value]) => `${name}=${value.toFixed(6)}`)
    .join(" ")}`,
);
console.log(`[wabou] wrote ${comparisonPng}`);

const limits = {
  global: 0.003,
  solid: 0.0001,
  alpha: 0.0001,
  roundedClip: 0.01,
  shadow: 0.01,
  translucent: 0.005,
} as const;
const regressions = [
  ...(normalizedRmse > limits.global
    ? [`global=${normalizedRmse.toFixed(6)} > ${limits.global}`]
    : []),
  ...Object.entries(regionRmse).flatMap(([name, value]) => {
    const limit = limits[name as keyof typeof regionRmse];
    return value > limit ? [`${name}=${value.toFixed(6)} > ${limit}`] : [];
  }),
];
if (regressions.length > 0) {
  throw new Error(
    `browser paint comparison regressed: ${regressions.join(", ")}`,
  );
}
