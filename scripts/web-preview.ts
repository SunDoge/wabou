import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const command = process.argv[2] ?? "dev";
if (!["dev", "build"].includes(command))
  throw new Error("Usage: bun scripts/web-preview.ts [dev|build]");
async function run(args: string[]) {
  const child = Bun.spawn(args, {
    cwd: root,
    stdout: "inherit",
    stderr: "inherit",
  });
  const code = await child.exited;
  if (code !== 0) process.exit(code);
}
await run([
  "cargo",
  "build",
  "--release",
  "-p",
  "wabou-web",
  "--target",
  "wasm32-unknown-unknown",
]);
const pkg = resolve(root, "target/web-preview/pkg");
await mkdir(pkg, { recursive: true });
await run([
  "wasm-bindgen",
  resolve(root, "target/wasm32-unknown-unknown/release/wabou_web.wasm"),
  "--target",
  "web",
  "--out-dir",
  pkg,
]);
await run([
  "bun",
  "x",
  "vite",
  ...(command === "build" ? ["build"] : []),
  "--config",
  "examples/web-preview/wabou.config.ts",
]);
