#!/usr/bin/env bun

import { mkdir, mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

interface TextRow {
  text: string;
  size: number;
  weight: number;
  x: number;
  baseline: number;
}

interface TextCases {
  width: number;
  height: number;
  background: string;
  foreground: string;
  fontFamily: string;
  scales: number[];
  rows: TextRow[];
}

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const valueAfter = (name: string) => {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : undefined;
};
const output = resolve(
  valueAfter("--out") ?? join(root, "reports/text-quality"),
);
const manifest = resolve(
  valueAfter("--manifest") ?? join(root, "tools/text-quality/cases.json"),
);
const hinting = valueAfter("--hinting") ?? "slight";
const antialias = valueAfter("--antialias") ?? "gray";
const subpixelPositions = !args.includes("--integer-positions");

async function run(command: string[], capture = false): Promise<string> {
  const process = Bun.spawn(command, {
    cwd: root,
    stdout: capture ? "pipe" : "inherit",
    stderr: capture ? "pipe" : "inherit",
  });
  const [stdout, stderr, status] = await Promise.all([
    capture ? new Response(process.stdout).text() : "",
    capture ? new Response(process.stderr).text() : "",
    process.exited,
  ]);
  if (status !== 0) {
    throw new Error(`${command.join(" ")} failed (${status})\n${stderr}`);
  }
  return stdout.trim();
}

async function rmse(left: string, right: string): Promise<string> {
  const process = Bun.spawn(
    ["compare", "-metric", "RMSE", left, right, "null:"],
    { cwd: root, stdout: "pipe", stderr: "pipe" },
  );
  const [stderr, status] = await Promise.all([
    new Response(process.stderr).text(),
    process.exited,
  ]);
  // ImageMagick uses status 1 to mean that the images differ.
  if (status > 1) throw new Error(`ImageMagick compare failed (${status})`);
  return stderr.trim();
}

await mkdir(output, { recursive: true });
const cases = (await Bun.file(manifest).json()) as TextCases;
const scratch = await mkdtemp(join(tmpdir(), "wabou-text-quality-"));
const pkgConfig = (
  await run(["pkg-config", "--cflags", "--libs", "pangocairo"], true)
)
  .split(/\s+/)
  .filter(Boolean);
const pangoHelper = join(scratch, "pango-reference");
await run([
  "cc",
  join(root, "tools/text-quality/pango_reference.c"),
  "-O2",
  "-o",
  pangoHelper,
  ...pkgConfig,
]);

await run([
  "cargo",
  "run",
  "-p",
  "wabou-shell-vello",
  "--example",
  "text_quality_probe",
  "--",
  output,
  manifest,
]);

const versions = {
  pango: await run(["pkg-config", "--modversion", "pango"], true),
  cairo: await run(["pkg-config", "--modversion", "cairo"], true),
  freetype: await run(["pkg-config", "--modversion", "freetype2"], true),
  chrome: await run(["google-chrome", "--version"], true),
  fontMatch: await run(["fc-match", cases.fontFamily], true),
};
const comparisons: Record<string, Record<string, string>> = {};

for (const scale of cases.scales) {
  const pangoPng = join(output, `pango-text-quality-${scale}x.png`);
  const pangoReport = join(output, `pango-text-quality-${scale}x.txt`);
  const rowArgs = cases.rows.flatMap((row) => [
    String(row.x),
    String(row.baseline),
    String(row.size),
    String(row.weight),
    row.text,
  ]);
  await run([
    pangoHelper,
    pangoPng,
    pangoReport,
    String(cases.width),
    String(cases.height),
    String(scale),
    cases.fontFamily,
    cases.background,
    cases.foreground,
    hinting,
    antialias,
    subpixelPositions ? "1" : "0",
    ...rowArgs,
  ]);

  const html = `<!doctype html><meta charset="utf-8"><style>
html,body{margin:0;width:${cases.width}px;height:${cases.height}px;overflow:hidden;background:${cases.background}}
canvas{display:block;width:${cases.width}px;height:${cases.height}px}
</style><canvas width="${cases.width * scale}" height="${cases.height * scale}"></canvas><script>
const canvas=document.querySelector('canvas');const cx=canvas.getContext('2d');
cx.scale(${scale},${scale});cx.fillStyle=${JSON.stringify(cases.background)};cx.fillRect(0,0,${cases.width},${cases.height});
cx.fillStyle=${JSON.stringify(cases.foreground)};cx.textBaseline='alphabetic';
const rows=${JSON.stringify(cases.rows)};
for(const row of rows){cx.font=row.weight+' '+row.size+'px '+${JSON.stringify(cases.fontFamily)};cx.fillText(row.text,row.x,row.baseline)}
</script>`;
  const htmlPath = join(scratch, `skia-${scale}x.html`);
  await writeFile(htmlPath, html);
  const skiaPng = join(output, `skia-text-quality-${scale}x.png`);
  await run([
    "google-chrome",
    "--headless=new",
    "--disable-gpu",
    "--hide-scrollbars",
    "--no-first-run",
    "--no-default-browser-check",
    `--user-data-dir=${join(scratch, `chrome-${scale}`)}`,
    `--force-device-scale-factor=${scale}`,
    `--window-size=${cases.width},${cases.height}`,
    `--screenshot=${skiaPng}`,
    pathToFileURL(htmlPath).href,
  ]);

  const wabouPng = join(output, `wabou-text-quality-${scale}x.png`);
  const panelWidth = cases.width * scale;
  const headerHeight = 32 * scale;
  const panels: string[] = [];
  for (const [name, image] of [
    ["Wabou · Parley / Vello", wabouPng],
    [`System · Pango ${hinting} / ${antialias}`, pangoPng],
    ["Skia · Chrome Canvas2D", skiaPng],
  ] as const) {
    const safeName = name.split(" · ")[0].toLowerCase();
    const header = join(scratch, `${safeName}-header-${scale}x.png`);
    const panel = join(scratch, `${safeName}-panel-${scale}x.png`);
    await run([
      "magick",
      "-size",
      `${panelWidth}x${headerHeight}`,
      `xc:${cases.background}`,
      "-fill",
      cases.foreground,
      "-gravity",
      "center",
      "-pointsize",
      String(13 * scale),
      "-annotate",
      "+0+0",
      name,
      header,
    ]);
    await run(["magick", header, image, "-append", panel]);
    panels.push(panel);
  }
  await run([
    "magick",
    ...panels,
    "+append",
    join(output, `comparison-${scale}x.png`),
  ]);
  comparisons[`${scale}x`] = {
    pangoRmse: await rmse(wabouPng, pangoPng),
    skiaRmse: await rmse(wabouPng, skiaPng),
  };
  for (const [name, reference] of [
    ["pango", pangoPng],
    ["skia", skiaPng],
  ] as const) {
    await run([
      "compare",
      "-compose",
      "src",
      wabouPng,
      reference,
      join(output, `difference-wabou-${name}-${scale}x.png`),
    ]).catch(() => undefined);
  }
}

await writeFile(
  join(output, "metadata.json"),
  `${JSON.stringify(
    {
      manifest,
      hinting,
      antialias,
      subpixelPositions,
      versions,
      comparisons,
    },
    null,
    2,
  )}\n`,
);
console.log(`Text quality report: ${output}`);
