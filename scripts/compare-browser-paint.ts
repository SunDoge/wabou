import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import solid from "@solidjs/vite-plugin";
import UnoCSS from "unocss/vite";
import { build } from "vite";

type Rect = { x: number; y: number; width: number; height: number };
type BrowserNode = {
  name: string | null;
  rect: Rect;
  style: { background: string; color: string };
};
type NativeNode = {
  attrs: [string, string][];
  rect: Rect;
  computed: { background: string | null; textColor: string };
};

const root = resolve(import.meta.dir, "..");
const argv = process.argv.slice(2);
const argument = (name: string, fallback: string): string => {
  const index = argv.indexOf(name);
  return index >= 0 ? (argv[index + 1] ?? fallback) : fallback;
};
const positionalOutput = argv[0]?.startsWith("--") ? undefined : argv[0];
const output = resolve(
  argument("--out", positionalOutput ?? "target/render-reference"),
);
const app = argument("--app", "apps/gallery");
const fixture = argument("--fixture", "foundations/paint-reference");
const width = Number(argument("--width", "640"));
const height = Number(argument("--height", "420"));
const layoutTolerance = Number(argument("--layout-tolerance", "0.51"));
const pixelTolerance = Number(argument("--pixel-tolerance", "0.01"));
const regionTolerance = Number(argument("--region-tolerance", "0.01"));
const wabou =
  process.env.WABOU_RENDER_COMMAND ?? resolve(root, "target/release/wabou");
const chrome =
  process.env.CHROME ??
  ["google-chrome", "chromium", "chromium-browser"].find((candidate) =>
    Bun.which(candidate),
  );
const magick = process.env.MAGICK ?? Bun.which("magick");

if (
  !Number.isFinite(width) ||
  !Number.isFinite(height) ||
  !Number.isFinite(layoutTolerance) ||
  !Number.isFinite(pixelTolerance) ||
  !Number.isFinite(regionTolerance)
) {
  throw new Error("viewport dimensions and tolerances must be finite numbers");
}
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
const browserRoot = resolve(root, "tests/render-reference");
const browserDist = resolve(output, "browser");

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

async function capture(command: string, args: string[]): Promise<string> {
  const child = Bun.spawn([command, ...args], {
    cwd: root,
    stdout: "pipe",
    stderr: "inherit",
  });
  const stdout = await new Response(child.stdout).text();
  const status = await child.exited;
  if (status !== 0) throw new Error(`${command} exited with status ${status}`);
  return stdout;
}

await build({
  root: browserRoot,
  base: "./",
  configFile: false,
  logLevel: "warn",
  define: {
    __WABOU_COMPARE_WIDTH__: JSON.stringify(width),
    __WABOU_COMPARE_HEIGHT__: JSON.stringify(height),
  },
  plugins: [solid(), UnoCSS(resolve(browserRoot, "uno.config.ts"))],
  resolve: {
    alias: [
      {
        find: /^@wabou\/ui$/,
        replacement: resolve(browserRoot, "web-ui.tsx"),
      },
    ],
  },
  build: { outDir: browserDist, emptyOutDir: true },
});

await run(wabou, [
  "render",
  app,
  "--fixture",
  fixture,
  "--wait-ms",
  "0",
  "--out",
  nativePng,
  "--snapshot",
  nativeSnapshot,
]);

const browserUrl = pathToFileURL(resolve(browserDist, "index.html")).href;
const chromeBase = [
  "--headless=new",
  "--no-sandbox",
  "--disable-dev-shm-usage",
  "--allow-file-access-from-files",
  "--hide-scrollbars",
  "--force-device-scale-factor=1",
  `--window-size=${width},${height}`,
  "--virtual-time-budget=1000",
];
const dom = await capture(chrome, [...chromeBase, "--dump-dom", browserUrl]);
const encodedBrowserSnapshot = dom.match(
  /data-wabou-snapshot="(?<snapshot>[^"]+)"/,
)?.groups?.snapshot;
if (!encodedBrowserSnapshot) {
  throw new Error("Chromium did not produce the cross-render layout snapshot");
}
const browserNodes = JSON.parse(
  Buffer.from(encodedBrowserSnapshot, "base64").toString("utf8"),
) as BrowserNode[];

await run(chrome, [...chromeBase, `--screenshot=${browserPng}`, browserUrl]);

const nativeReport = (await Bun.file(nativeSnapshot).json()) as {
  nodes: NativeNode[];
};
const nativeNodes = new Map(
  nativeReport.nodes.flatMap((node) => {
    const name = node.attrs.find(([key]) => key === "aria-label")?.[1];
    return name?.startsWith("compare/") ? [[name, node] as const] : [];
  }),
);

function normalizeBrowserColor(value: string): string | null {
  const match = value.match(
    /^rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([0-9.]+))?\)$/,
  );
  if (!match) return value.toLowerCase();
  const alpha = Math.round(Number(match[4] ?? 1) * 255);
  if (alpha === 0) return null;
  return `#${[match[1], match[2], match[3]]
    .map((part) => Number(part).toString(16).padStart(2, "0"))
    .join("")}${alpha === 255 ? "" : alpha.toString(16).padStart(2, "0")}`;
}

const geometry: Record<string, Record<keyof Rect, number>> = {};
const colors: Record<
  string,
  { wabou: string | null; chromium: string | null }
> = {};
const comparedBrowserNodes: BrowserNode[] = [];
const contractRegressions: string[] = [];
for (const browserNode of browserNodes) {
  if (!browserNode.name?.startsWith("compare/")) continue;
  const nativeNode = nativeNodes.get(browserNode.name);
  if (!nativeNode) {
    contractRegressions.push(
      `${browserNode.name}: missing from Wabou snapshot`,
    );
    continue;
  }
  comparedBrowserNodes.push(browserNode);
  geometry[browserNode.name] = {
    x: nativeNode.rect.x - browserNode.rect.x,
    y: nativeNode.rect.y - browserNode.rect.y,
    width: nativeNode.rect.width - browserNode.rect.width,
    height: nativeNode.rect.height - browserNode.rect.height,
  };
  for (const [field, delta] of Object.entries(geometry[browserNode.name])) {
    if (Math.abs(delta) > layoutTolerance) {
      contractRegressions.push(
        `${browserNode.name}.${field}: delta ${delta.toFixed(3)}px exceeds ${layoutTolerance}px`,
      );
    }
  }
  const browserBackground = normalizeBrowserColor(browserNode.style.background);
  const nativeBackground =
    nativeNode.computed.background?.toLowerCase() ?? null;
  colors[browserNode.name] = {
    wabou: nativeBackground,
    chromium: browserBackground,
  };
  if (nativeBackground !== browserBackground) {
    contractRegressions.push(
      `${browserNode.name}.background: Wabou ${nativeBackground} != Chromium ${browserBackground}`,
    );
  }
  nativeNodes.delete(browserNode.name);
}
for (const name of nativeNodes.keys()) {
  contractRegressions.push(`${name}: missing from Chromium snapshot`);
}

const rmseOutput = await run(
  magick,
  ["compare", "-metric", "RMSE", nativePng, browserPng, differencePng],
  true,
);
const normalizedRmse = Number(
  rmseOutput.match(/\((?<normalized>[0-9.]+)\)/)?.groups?.normalized ??
    Number.NaN,
);
const regionRmse: Record<string, number> = {};
for (const node of comparedBrowserNodes) {
  if (!node.name || node.name === "compare/root") continue;
  const left = Math.max(0, Math.floor(node.rect.x));
  const top = Math.max(0, Math.floor(node.rect.y));
  const right = Math.min(width, Math.ceil(node.rect.x + node.rect.width));
  const bottom = Math.min(height, Math.ceil(node.rect.y + node.rect.height));
  if (right <= left || bottom <= top) continue;
  const geometry = `${right - left}x${bottom - top}+${left}+${top}`;
  const slug = node.name.replace(/^compare\//, "").replace(/[^a-z0-9-]/gi, "-");
  const nativeCrop = resolve(output, `${slug}-wabou.png`);
  const browserCrop = resolve(output, `${slug}-chromium.png`);
  const differenceCrop = resolve(output, `${slug}-difference.png`);
  await run(magick, [nativePng, "-crop", geometry, "+repage", nativeCrop]);
  await run(magick, [browserPng, "-crop", geometry, "+repage", browserCrop]);
  const metric = await run(
    magick,
    ["compare", "-metric", "RMSE", nativeCrop, browserCrop, differenceCrop],
    true,
  );
  const value = Number(
    metric.match(/\((?<normalized>[0-9.]+)\)/)?.groups?.normalized ??
      Number.NaN,
  );
  regionRmse[node.name] = value;
  if (value > regionTolerance) {
    contractRegressions.push(
      `${node.name}: pixel RMSE ${value.toFixed(6)} exceeds ${regionTolerance}`,
    );
  }
}
await run(magick, [
  nativePng,
  browserPng,
  differencePng,
  "+append",
  comparisonPng,
]);
if (normalizedRmse > pixelTolerance) {
  contractRegressions.push(
    `global pixel RMSE ${normalizedRmse.toFixed(6)} exceeds ${pixelTolerance}`,
  );
}

const report = {
  version: 2,
  app,
  fixture,
  viewport: { width, height, scaleFactor: 1 },
  normalizedRmse,
  layoutTolerance,
  pixelTolerance,
  regionTolerance,
  geometry,
  colors,
  regionRmse,
  regressions: contractRegressions,
  note: "The same Solid TSX and Wabou UnoCSS preset render through native Wabou primitives and a DOM adapter. Text rasterization and platform-specific compositing require separate references.",
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
  `[wabou] cross-render comparison RMSE=${normalizedRmse.toFixed(6)} nodes=${Object.keys(geometry).length}`,
);
console.log(`[wabou] wrote ${comparisonPng}`);

if (contractRegressions.length > 0) {
  throw new Error(
    `cross-render comparison regressed:\n  - ${contractRegressions.join("\n  - ")}`,
  );
}
