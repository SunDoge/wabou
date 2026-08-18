import { readdir, readFile, writeFile } from "node:fs/promises";
import { relative, resolve, sep } from "node:path";

const encoder = new TextEncoder();
const MASK = 0xffffffffffffffffn;
const PRIME = 0x100000001b3n;

async function sourceFiles(root: string): Promise<string[]> {
  const source = resolve(root, "src");
  const entries = await readdir(source, {
    recursive: true,
    withFileTypes: true,
  });
  return entries
    .filter((entry) => entry.isFile())
    .map((entry) => resolve(entry.parentPath, entry.name))
    .sort((left, right) => left.localeCompare(right));
}

async function sourceHash(root: string): Promise<string> {
  let hash = 0xcbf29ce484222325n;
  const update = (bytes: Uint8Array) => {
    for (const byte of bytes) hash = ((hash ^ BigInt(byte)) * PRIME) & MASK;
  };
  for (const file of await sourceFiles(root)) {
    update(encoder.encode(relative(root, file).split(sep).join("/")));
    update(Uint8Array.of(0));
    update(await readFile(file));
    update(Uint8Array.of(0xff));
  }
  return hash.toString(16).padStart(16, "0");
}

const packagesRoot = resolve(import.meta.dir, "../packages");
const hashes: Record<string, string> = {};
for (const entry of await readdir(packagesRoot, { withFileTypes: true })) {
  if (!entry.isDirectory()) continue;
  const root = resolve(packagesRoot, entry.name);
  const manifest = JSON.parse(
    await readFile(resolve(root, "package.json"), "utf8"),
  );
  if (typeof manifest.name !== "string" || !manifest.name.startsWith("@wabou/"))
    continue;
  hashes[manifest.name] = await sourceHash(root);
}
await writeFile(
  resolve(packagesRoot, ".wabou-source-hashes.json"),
  `${JSON.stringify(hashes, Object.keys(hashes).sort(), 2)}\n`,
);
