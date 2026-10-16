import { mkdir, rm } from "node:fs/promises";
import { relative, resolve, sep } from "node:path";
import { gzipSync } from "node:zlib";
import { build, type Plugin } from "vite";

interface ModuleContribution {
  id: string;
  renderedBytes: number;
}

interface ChunkReport {
  fileName: string;
  rawBytes: number;
  gzipBytes: number;
  modules: ModuleContribution[];
}

const repository = resolve(import.meta.dirname, "..");
const reportRoot = resolve(repository, "target/wabou-reports/js-bundle");

function displayModule(id: string): string {
  const clean = id.replaceAll("\\0", "").split("?")[0];
  const marker = `${sep}node_modules${sep}`;
  const dependency = clean.lastIndexOf(marker);
  if (dependency >= 0)
    return `node_modules/${clean.slice(dependency + marker.length)}`;
  const local = relative(repository, clean);
  return local.startsWith("..") ? clean : local;
}

function kib(bytes: number): string {
  return `${(bytes / 1024).toFixed(1)} KiB`;
}

async function reportApplication(input: string) {
  const appRoot = resolve(repository, input);
  const name = appRoot.split(sep).at(-1) ?? "application";
  const output = resolve(reportRoot, `${name}-dist`);
  const reportPath = resolve(reportRoot, `${name}.json`);
  const chunks: ChunkReport[] = [];
  const plugin: Plugin = {
    name: "wabou-js-bundle-report",
    generateBundle(_options, bundle) {
      for (const value of Object.values(bundle)) {
        if (value.type !== "chunk") continue;
        const chunk = value;
        const raw = Buffer.byteLength(chunk.code);
        chunks.push({
          fileName: chunk.fileName,
          rawBytes: raw,
          gzipBytes: gzipSync(chunk.code, { level: 9 }).byteLength,
          modules: Object.entries(chunk.modules)
            .map(([id, module]) => ({
              id: displayModule(id),
              renderedBytes: module.renderedLength,
            }))
            .sort((left, right) => right.renderedBytes - left.renderedBytes),
        });
      }
    },
  };

  await rm(output, { recursive: true, force: true });
  const previousDirectory = process.cwd();
  const previousOutDir = process.env.WABOU_OUT_DIR;
  const previousSourceMap = process.env.WABOU_SOURCE_MAP;
  try {
    process.chdir(appRoot);
    process.env.WABOU_OUT_DIR = output;
    process.env.WABOU_SOURCE_MAP = "false";
    await build({
      root: appRoot,
      configFile: resolve(appRoot, "wabou.config.ts"),
      mode: "production",
      logLevel: "warn",
      plugins: [plugin],
    });
  } finally {
    process.chdir(previousDirectory);
    if (previousOutDir === undefined) delete process.env.WABOU_OUT_DIR;
    else process.env.WABOU_OUT_DIR = previousOutDir;
    if (previousSourceMap === undefined) delete process.env.WABOU_SOURCE_MAP;
    else process.env.WABOU_SOURCE_MAP = previousSourceMap;
  }

  const modules = new Map<string, ModuleContribution>();
  for (const chunk of chunks) {
    for (const module of chunk.modules) {
      const current = modules.get(module.id);
      if (current) {
        current.renderedBytes += module.renderedBytes;
      } else {
        modules.set(module.id, { ...module });
      }
    }
  }
  const sortedModules = [...modules.values()].sort(
    (left, right) => right.renderedBytes - left.renderedBytes,
  );
  const report = {
    schemaVersion: 1,
    application: relative(repository, appRoot),
    generatedAt: new Date().toISOString(),
    totalRawBytes: chunks.reduce((sum, chunk) => sum + chunk.rawBytes, 0),
    totalGzipBytes: chunks.reduce((sum, chunk) => sum + chunk.gzipBytes, 0),
    chunks,
    modules: sortedModules,
  };
  await Bun.write(reportPath, `${JSON.stringify(report, null, 2)}\n`);

  console.log(
    `\n${report.application}: ${kib(report.totalRawBytes)} raw, ${kib(report.totalGzipBytes)} gzip`,
  );
  console.log("| Module | Rendered | Share |");
  console.log("| --- | ---: | ---: |");
  for (const module of sortedModules.slice(0, 20)) {
    const share =
      report.totalRawBytes === 0
        ? 0
        : (module.renderedBytes / report.totalRawBytes) * 100;
    console.log(
      `| \`${module.id}\` | ${kib(module.renderedBytes)} | ${share.toFixed(1)}% |`,
    );
  }
  console.log(`Report: ${relative(repository, reportPath)}`);
}

await mkdir(reportRoot, { recursive: true });
const applications = process.argv.slice(2);
for (const application of applications.length > 0
  ? applications
  : ["apps/meow-starter", "apps/gallery"]) {
  await reportApplication(application);
}
