import { appendFile } from "node:fs/promises";
import { relative, resolve } from "node:path";
import { gzipSync } from "node:zlib";

const root = resolve(import.meta.dirname, "..");
const paths = process.argv.slice(2);

if (paths.length === 0) {
  throw new Error("usage: report-artifact-sizes.ts <artifact> [...artifact]");
}

function kib(bytes: number): string {
  return `${(bytes / 1024).toFixed(1)} KiB`;
}

const rows: string[] = [];
for (const input of paths) {
  const path = resolve(root, input);
  const file = Bun.file(path);
  if (!(await file.exists())) {
    throw new Error(`artifact does not exist: ${relative(root, path)}`);
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  rows.push(
    `| \`${relative(root, path)}\` | ${kib(bytes.byteLength)} | ${kib(gzipSync(bytes, { level: 9 }).byteLength)} |`,
  );
}

const report = [
  "## JavaScript artifact sizes",
  "",
  "| Artifact | Raw | gzip -9 |",
  "| --- | ---: | ---: |",
  ...rows,
  "",
].join("\n");

console.log(report);

const summary = process.env.GITHUB_STEP_SUMMARY;
if (summary) {
  await appendFile(summary, report);
}
