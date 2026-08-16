import { readdir } from "node:fs/promises";
import { join } from "node:path";

const root = new URL("..", import.meta.url).pathname;
const internalPackages = new Set([
  "@wabou/protocol",
  "@wabou/solid-renderer",
  "@wabou/style",
  "@wabou/style-compiler",
  "@wabou/unocss-preset",
]);

interface Manifest {
  name: string;
  version: string;
  description?: string;
  license?: string;
  repository?: { type?: string; url?: string; directory?: string };
  private?: boolean;
  publishConfig?: { access?: string };
  wabou?: { stability?: string };
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
}

async function manifest(path: string): Promise<Manifest> {
  return Bun.file(path).json();
}

const packageDirs = await readdir(join(root, "packages"), {
  withFileTypes: true,
});
const packageManifestPaths = packageDirs
  .filter((entry) => entry.isDirectory())
  .map((entry) => join(root, "packages", entry.name, "package.json"));
const packages = await Promise.all(
  packageManifestPaths
    .filter((path) => Bun.file(path).size > 0)
    .map((path) => manifest(path)),
);
const versions = new Set(packages.map((entry) => entry.version));
if (versions.size !== 1) {
  throw new Error(
    `@wabou packages must share one version; found ${[...versions]}`,
  );
}
const packageVersion = [...versions][0];
const cargoManifest = await Bun.file(join(root, "Cargo.toml")).text();
const cargoVersion = cargoManifest.match(
  /^version\s*=\s*"([^"]+)"\s*$/m,
)?.[1];
if (!cargoVersion) {
  throw new Error("Cargo.toml must declare workspace.package.version");
}
if (cargoVersion !== packageVersion) {
  throw new Error(
    `Rust and JavaScript release versions differ: ${cargoVersion} != ${packageVersion}`,
  );
}
const changelog = await Bun.file(join(root, "CHANGELOG.md")).text();
if (!changelog.includes(`## ${packageVersion} -`)) {
  throw new Error(`CHANGELOG.md has no ${packageVersion} release heading`);
}
for (const entry of packages) {
  if (entry.private)
    throw new Error(`${entry.name} cannot be published while private`);
  if (entry.publishConfig?.access !== "public") {
    throw new Error(`${entry.name} must publish with public npm access`);
  }
  if (!entry.description) {
    throw new Error(`${entry.name} must have a package description`);
  }
  if (entry.license !== "Apache-2.0") {
    throw new Error(`${entry.name} must declare the Apache-2.0 license`);
  }
  if (
    entry.repository?.type !== "git" ||
    entry.repository.url !== "git+https://github.com/SunDoge/wabou.git" ||
    !entry.repository.directory
  ) {
    throw new Error(`${entry.name} must link to its repository directory`);
  }
  const internal = internalPackages.has(entry.name);
  if ((entry.wabou?.stability === "internal") !== internal) {
    throw new Error(`${entry.name} has incorrect Wabou stability metadata`);
  }
}

const appDirs = await readdir(join(root, "apps"), { withFileTypes: true });
for (const directory of appDirs.filter((entry) => entry.isDirectory())) {
  const entry = await manifest(
    join(root, "apps", directory.name, "package.json"),
  );
  const declared = { ...entry.dependencies, ...entry.devDependencies };
  for (const dependency of internalPackages) {
    if (dependency in declared) {
      throw new Error(
        `${entry.name} directly depends on internal ${dependency}`,
      );
    }
  }
}

const sourceGlob = new Bun.Glob("apps/**/*.{ts,tsx}");
for await (const path of sourceGlob.scan({ cwd: root, onlyFiles: true })) {
  if (path.includes("/generated/") || path.endsWith("custom-elements.d.ts")) {
    continue;
  }
  const source = await Bun.file(join(root, path)).text();
  for (const dependency of internalPackages) {
    if (
      source.includes(`\"${dependency}`) ||
      source.includes(`'${dependency}`)
    ) {
      throw new Error(`${path} directly imports internal ${dependency}`);
    }
  }
}

console.log(
  `verified ${packages.length} aligned packages and Rust workspace at ${packageVersion}, plus ${appDirs.length} app manifests`,
);
