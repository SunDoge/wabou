import { readdir } from "node:fs/promises";
import { join } from "node:path";
import ts from "typescript";

const root = new URL("..", import.meta.url).pathname;
// Retired implementation package names stay forbidden so new code cannot
// accidentally recreate the package graph that @wabou/ui replaced.
const retiredPackages = new Set([
  "@wabou/animation",
  "@wabou/components",
  "@wabou/primitives",
  "@wabou/router",
]);
const applicationLayerPackages = new Set(["@wabou/core"]);
const applicationDependencyExceptions = new Map([
  ["@wabou/devtools-app", new Set(["@wabou/core"])],
  ["@wabou/gallery", new Set(["@wabou/core"])],
]);
const rustApplicationDependencyExceptions = new Map([
  ["devtools", new Set(["wabou-devtools"])],
  ["terminal", new Set(["wabou-terminal"])],
]);

function cargoDependencies(source: string): Set<string> {
  const dependencies = new Set<string>();
  let dependencySection = false;
  for (const line of source.split(/\r?\n/)) {
    const section = line.match(/^\s*\[([^\]]+)]\s*$/)?.[1];
    if (section) {
      dependencySection =
        section === "dependencies" ||
        section === "dev-dependencies" ||
        section === "build-dependencies" ||
        section.endsWith(".dependencies") ||
        section.endsWith(".dev-dependencies") ||
        section.endsWith(".build-dependencies");
      const tableDependency = section.match(
        /^(?:dev-|build-)?dependencies\.([A-Za-z0-9_-]+)$/,
      )?.[1];
      if (tableDependency) dependencies.add(tableDependency);
      continue;
    }
    if (!dependencySection) continue;
    const dependency = line.match(/^\s*([A-Za-z0-9_-]+)\s*=/)?.[1];
    if (dependency) dependencies.add(dependency);
  }
  return dependencies;
}

async function verifyRustApplication(
  directory: string,
  allowed: Set<string>,
): Promise<void> {
  const manifestPath = join(directory, "Cargo.toml");
  const dependencies = cargoDependencies(await Bun.file(manifestPath).text());
  if (!dependencies.has("wabou")) {
    throw new Error(`${manifestPath} must depend on the wabou facade`);
  }
  for (const dependency of dependencies) {
    if (
      dependency.startsWith("wabou-") &&
      dependency !== "wabou" &&
      !allowed.has(dependency)
    ) {
      throw new Error(
        `${manifestPath} directly depends on internal ${dependency}; use wabou`,
      );
    }
  }
}

interface Manifest {
  name: string;
  version: string;
  description?: string;
  license?: string;
  repository?: { type?: string; url?: string; directory?: string };
  private?: boolean;
  publishConfig?: { access?: string };
  files?: string[];
  exports?: unknown;
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
  optionalDependencies?: Record<string, string>;
  peerDependencies?: Record<string, string>;
}

interface ConditionalExport {
  "wabou-source"?: string;
  types?: string;
  import?: string;
  default?: string;
}

async function manifest(path: string): Promise<Manifest> {
  return Bun.file(path).json();
}

const packageDirs = await readdir(join(root, "packages"), {
  withFileTypes: true,
});
const packageManifestPaths = (
  await Promise.all(
    packageDirs
      .filter((entry) => entry.isDirectory())
      .map(async (entry) => {
        const path = join(root, "packages", entry.name, "package.json");
        return (await Bun.file(path).exists()) ? path : undefined;
      }),
  )
).filter((path): path is string => path !== undefined);
const packages = await Promise.all(
  packageManifestPaths.map((path) => manifest(path)),
);
const versions = new Set(packages.map((entry) => entry.version));
if (versions.size !== 1) {
  throw new Error(
    `@wabou packages must share one version; found ${[...versions]}`,
  );
}
const packageVersion = [...versions][0];
const cargoManifest = await Bun.file(join(root, "Cargo.toml")).text();
const cargoVersion = cargoManifest.match(/^version\s*=\s*"([^"]+)"\s*$/m)?.[1];
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
  if (retiredPackages.has(entry.name)) {
    throw new Error(`${entry.name} was folded into @wabou/ui and must not return`);
  }
  if (entry.private) {
    throw new Error(
      `${entry.name} is source-only; use a directory inside its owning public package`,
    );
  }
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
  if (!entry.files?.includes("dist")) {
    throw new Error(`${entry.name} must publish its dist directory`);
  }
  const publishedDependencies = {
    ...entry.dependencies,
    ...entry.optionalDependencies,
    ...entry.peerDependencies,
  };
  for (const dependency of retiredPackages) {
    if (dependency in publishedDependencies) {
      throw new Error(
        `${entry.name} cannot depend on retired package ${dependency}`,
      );
    }
  }
}

async function verifyExportTargets(
  packageRoot: string,
  packageName: string,
  value: unknown,
): Promise<void> {
  if (typeof value === "string") {
    if (!(await Bun.file(join(packageRoot, value)).exists())) {
      throw new Error(`${packageName} export target does not exist: ${value}`);
    }
    return;
  }
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${packageName} has an invalid exports entry`);
  }
  for (const target of Object.values(value as ConditionalExport)) {
    await verifyExportTargets(packageRoot, packageName, target);
  }
}

function dependencyName(specifier: string): string | undefined {
  if (
    specifier.startsWith(".") ||
    specifier.startsWith("/") ||
    specifier.startsWith("#") ||
    specifier.startsWith("node:")
  ) {
    return undefined;
  }
  const parts = specifier.split("/");
  return specifier.startsWith("@") ? parts.slice(0, 2).join("/") : parts[0];
}

function importedSpecifiers(source: string, path: string): string[] {
  const sourceFile = ts.createSourceFile(
    path,
    source,
    ts.ScriptTarget.Latest,
    false,
    path.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
  );
  const specifiers: string[] = [];
  const visit = (node: ts.Node) => {
    if (
      (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) &&
      node.moduleSpecifier &&
      ts.isStringLiteral(node.moduleSpecifier)
    ) {
      specifiers.push(node.moduleSpecifier.text);
    } else if (
      ts.isCallExpression(node) &&
      node.expression.kind === ts.SyntaxKind.ImportKeyword &&
      node.arguments.length === 1 &&
      ts.isStringLiteral(node.arguments[0])
    ) {
      specifiers.push(node.arguments[0].text);
    } else if (
      ts.isImportTypeNode(node) &&
      ts.isLiteralTypeNode(node.argument) &&
      ts.isStringLiteral(node.argument.literal)
    ) {
      specifiers.push(node.argument.literal.text);
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return specifiers;
}

for (const manifestPath of packageManifestPaths) {
  const entry = await manifest(manifestPath);
  const packageRoot = join(manifestPath, "..");
  await verifyExportTargets(packageRoot, entry.name, entry.exports);
  const declared = {
    ...entry.dependencies,
    ...entry.optionalDependencies,
    ...entry.peerDependencies,
  };
  const sourceGlob = new Bun.Glob("src/**/*.{ts,tsx,js,mjs}");
  for await (const path of sourceGlob.scan({
    cwd: packageRoot,
    onlyFiles: true,
  })) {
    if (path.includes(".test.") || path.includes(".spec.")) continue;
    const source = await Bun.file(join(packageRoot, path)).text();
    for (const specifier of importedSpecifiers(source, path)) {
      const dependency = dependencyName(specifier);
      if (
        dependency &&
        dependency !== entry.name &&
        !(dependency in declared)
      ) {
        throw new Error(
          `${entry.name} imports undeclared dependency ${dependency} in ${path}`,
        );
      }
    }
  }
}

for (const manifestPath of packageManifestPaths) {
  const entry = await manifest(manifestPath);
  if (entry.private) continue;
  const packageRoot = join(manifestPath, "..");
  const distGlob = new Bun.Glob("dist/**/*.{mjs,d.mts}");
  for await (const path of distGlob.scan({
    cwd: packageRoot,
    onlyFiles: true,
  })) {
    const output = await Bun.file(join(packageRoot, path)).text();
    for (const dependency of retiredPackages) {
      if (
        output.includes(`"${dependency}`) ||
        output.includes(`'${dependency}`)
      ) {
        throw new Error(
          `${entry.name} published output ${path} imports retired ${dependency}`,
        );
      }
    }
  }
}

const appDirs = await readdir(join(root, "apps"), { withFileTypes: true });
for (const directory of appDirs.filter((entry) => entry.isDirectory())) {
  const entry = await manifest(
    join(root, "apps", directory.name, "package.json"),
  );
  const declared = { ...entry.dependencies, ...entry.devDependencies };
  for (const dependency of retiredPackages) {
    if (dependency in declared) {
      throw new Error(
        `${entry.name} directly depends on retired ${dependency}`,
      );
    }
  }
  const allowed = applicationDependencyExceptions.get(entry.name) ?? new Set();
  for (const dependency of applicationLayerPackages) {
    if (dependency in declared && !allowed.has(dependency)) {
      throw new Error(
        `${entry.name} directly depends on layered ${dependency}; use @wabou/ui`,
      );
    }
  }
  await verifyRustApplication(
    join(root, "apps", directory.name),
    rustApplicationDependencyExceptions.get(directory.name) ?? new Set(),
  );
}

await verifyRustApplication(join(root, "templates/basic"), new Set());

const sourceGlob = new Bun.Glob("apps/**/*.{ts,tsx}");
for await (const path of sourceGlob.scan({ cwd: root, onlyFiles: true })) {
  if (
    path.includes("/generated/") ||
    path.includes(".test.") ||
    path.endsWith("custom-elements.d.ts")
  ) {
    continue;
  }
  const source = await Bun.file(join(root, path)).text();
  for (const dependency of retiredPackages) {
    if (
      source.includes(`"${dependency}`) ||
      source.includes(`'${dependency}`)
    ) {
      throw new Error(`${path} directly imports retired ${dependency}`);
    }
  }
  for (const dependency of applicationLayerPackages) {
    if (
      source.includes(`"${dependency}`) ||
      source.includes(`'${dependency}`)
    ) {
      throw new Error(`${path} directly imports ${dependency}; use @wabou/ui`);
    }
  }
}

console.log(
  `verified ${packages.length} public packages and Rust workspace at ${packageVersion}, plus ${appDirs.length} app manifests`,
);
