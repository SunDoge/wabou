import { stat } from "node:fs/promises";
import { resolve } from "node:path";

const directory = import.meta.dir;
const root = resolve(directory, "../..");
const bundle = resolve(directory, "dist/bundle.js");
const bytes = (await stat(bundle)).size;
if (bytes > 150_000) throw new Error(`bundle grew to ${bytes} bytes`);

const child = Bun.spawn(
  [
    "cargo",
    "run",
    "--quiet",
    "--release",
    "-p",
    "wabou-runtime",
    "--example",
    "eval-bundle",
    "--",
    bundle,
    "globalThis.__wabouCodeMirrorExperiment",
  ],
  { cwd: root, stdout: "pipe", stderr: "inherit" },
);
const output = await new Response(child.stdout).text();
if ((await child.exited) !== 0) throw new Error("QuickJS probe failed");
const result = JSON.parse(output) as {
  edited: string;
  undoText: string;
  redoText: string;
  syntaxTreeChanged: boolean;
  highlights: { classes: string }[];
  stressDocumentLength: number;
  durationMs: number;
};

if (result.edited !== '{"enabled": true, "port": 8080}')
  throw new Error(`edit failed: ${result.edited}`);
if (result.undoText !== '{"enabled": true, "port": 9090}')
  throw new Error(`inverse transaction failed: ${result.undoText}`);
if (result.redoText !== result.edited)
  throw new Error(`redo transaction failed: ${result.redoText}`);
if (!result.syntaxTreeChanged) throw new Error("Lezer tree did not update");
for (const expected of ["property", "boolean", "number"]) {
  if (!result.highlights.some(({ classes }) => classes === expected))
    throw new Error(`missing ${expected} highlight range`);
}
if (result.stressDocumentLength !== 1_031)
  throw new Error(
    `transaction stress result was ${result.stressDocumentLength}`,
  );

console.log(
  JSON.stringify({
    passed: true,
    bundleBytes: bytes,
    quickJsDurationMs: result.durationMs,
    highlightRanges: result.highlights.length,
  }),
);
