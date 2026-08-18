import { readdir } from "node:fs/promises";
import { join } from "node:path";

const root = new URL("..", import.meta.url).pathname;
// Dependencies precede their consumers so a fresh version can be installed as
// soon as each public package reaches the registry.
const packageDirectories = [
  "protocol",
  "style",
  "solid-renderer",
  "core",
  "animation",
  "primitives",
  "components",
  "router",
  "terminal",
  "test",
  "vite",
];

const extraArguments = process.argv.slice(2);
const dryRun = extraArguments.includes("--dry-run");
const packageRoot = join(root, "packages");
const publicDirectories = (
  await readdir(packageRoot, { withFileTypes: true })
)
  .filter(
    (entry) =>
      entry.isDirectory() &&
      Bun.file(join(packageRoot, entry.name, "package.json")).size > 0,
  )
  .map((entry) => entry.name)
  .sort();
const configuredDirectories = [...packageDirectories].sort();
if (publicDirectories.join("\n") !== configuredDirectories.join("\n")) {
  const missing = publicDirectories.filter(
    (directory) => !packageDirectories.includes(directory),
  );
  const unknown = packageDirectories.filter(
    (directory) => !publicDirectories.includes(directory),
  );
  throw new Error(
    `publish order is incomplete (missing: ${missing.join(", ") || "none"}; unknown: ${unknown.join(", ") || "none"})`,
  );
}
for (const directory of packageDirectories) {
  const cwd = join(root, "packages", directory);
  const manifest = await Bun.file(join(cwd, "package.json")).json();
  console.log(
    `${dryRun ? "packing" : "publishing"} ${manifest.name}@${manifest.version}`,
  );
  const child = Bun.spawn(
    dryRun
      ? ["bun", "pm", "pack", "--dry-run"]
      : ["bun", "publish", "--access", "public", ...extraArguments],
    { cwd, stdin: "inherit", stdout: "inherit", stderr: "inherit" },
  );
  const status = await child.exited;
  if (status !== 0) process.exit(status);
}
