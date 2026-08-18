import { appendFile } from "node:fs/promises";

interface Report {
  kind: "headless";
  application: string;
  samples: number;
  nodeCount: number;
  viewport: { width: number; height: number; scaleFactor: number };
  medianMs: { build: number; scene: number };
}

const paths = process.argv.slice(2);
if (paths.length === 0) {
  throw new Error("usage: report-performance.ts <metrics.json> [...metrics.json]");
}

const rows: string[] = [];
for (const path of paths) {
  const report = (await Bun.file(path).json()) as Report;
  if (report.kind !== "headless" || report.samples < 1) {
    throw new Error(`invalid Wabou performance report: ${path}`);
  }
  rows.push(
    `| ${report.application} | ${report.nodeCount} | ${report.viewport.width}×${report.viewport.height}@${report.viewport.scaleFactor} | ${report.samples} | ${report.medianMs.build.toFixed(2)} | ${report.medianMs.scene.toFixed(2)} |`,
  );
}

const output = [
  "## Headless performance diagnostics",
  "",
  "These medians detect large retained-tree regressions. They exclude native surface presentation and are not an FPS claim.",
  "",
  "| Workload | Nodes | Viewport | Samples | Build ms | Scene ms |",
  "| --- | ---: | ---: | ---: | ---: | ---: |",
  ...rows,
  "",
].join("\n");

console.log(output);
if (process.env.GITHUB_STEP_SUMMARY) {
  await appendFile(process.env.GITHUB_STEP_SUMMARY, output);
}
