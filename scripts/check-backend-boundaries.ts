const RENDERER_LIBRARY_DEPENDENCIES = new Set(["vello", "vello_gpu", "winit"]);

const SHARED_PACKAGES = new Set([
  "wabou-bindgen",
  "wabou-database",
  "wabou-devtools",
  "wabou-host-api",
  "wabou-protocol",
  "wabou-runtime",
  "wabou-shell-api",
  "wabou-style",
]);

const NATIVE_RENDERER_PACKAGES = new Set([
  "wabou-renderer",
  "wabou-web",
  "wabou",
  "wabou-shell",
  "wabou-terminal-widget",
  "wabou-tray",
]);

// Ordinary workspace commands validate the framework's active dependency
// spine plus one representative application. Optional integrations and demo
// applications remain workspace members and are exercised by their owning
// application, behavior, and layout checks.
const FORMAL_VERIFICATION_PACKAGES = new Set([
  "gallery",
  "wabou",
  "wabou-bindgen",
  "wabou-cli",
  "wabou-devtools",
  "wabou-host-api",
  "wabou-motion",
  "wabou-protocol",
  "wabou-renderer",
  "wabou-runtime",
  "wabou-shell-api",
  "wabou-shell",
  "wabou-style",
  "wabou-telemetry",
  "wabou-web",
]);

interface CargoDependency {
  kind: "build" | "dev" | null;
  name: string;
  optional?: boolean;
  rename: string | null;
  uses_default_features?: boolean;
}

interface CargoPackage {
  dependencies: CargoDependency[];
  id?: string;
  name: string;
}

interface CargoMetadata {
  packages: CargoPackage[];
  workspace_default_members?: string[];
  workspace_members?: string[];
}

function formalWorkspacePackages(metadata: CargoMetadata): CargoPackage[] {
  const members = new Set(metadata.workspace_members ?? []);
  return metadata.packages.filter(
    (pkg) =>
      members.size === 0 || (pkg.id !== undefined && members.has(pkg.id)),
  );
}

function dependencyLabel(
  pkg: CargoPackage,
  dependency: CargoDependency,
): string {
  const visibleName = dependency.rename ?? dependency.name;
  const kind = dependency.kind ?? "normal";
  return `${pkg.name} -> ${visibleName} (${dependency.name}, ${kind})`;
}

function isRendererDependency(name: string): boolean {
  return (
    RENDERER_LIBRARY_DEPENDENCIES.has(name) ||
    NATIVE_RENDERER_PACKAGES.has(name)
  );
}

/** Backend-neutral crates must remain usable without either platform stack. */
export function sharedBoundaryViolations(metadata: CargoMetadata): string[] {
  const violations: string[] = [];
  for (const pkg of metadata.packages) {
    if (!SHARED_PACKAGES.has(pkg.name)) continue;
    for (const dependency of pkg.dependencies) {
      if (isRendererDependency(dependency.name)) {
        violations.push(dependencyLabel(pkg, dependency));
      }
    }
  }
  return violations.sort();
}

/** Backend entry points must keep the shared runtime renderer-neutral. */
export function backendFeatureIsolationViolations(
  metadata: CargoMetadata,
): string[] {
  const violations: string[] = [];
  for (const pkg of metadata.packages) {
    if (!NATIVE_RENDERER_PACKAGES.has(pkg.name)) continue;
    for (const dependency of pkg.dependencies) {
      if (
        dependency.name === "wabou-runtime" &&
        dependency.uses_default_features !== false
      ) {
        violations.push(
          `${dependencyLabel(pkg, dependency)} enables default backend features`,
        );
      }
    }
  }
  return violations.sort();
}

export function formalVerificationViolations(
  metadata: CargoMetadata,
): string[] {
  const defaultMembers = new Set(metadata.workspace_default_members ?? []);
  return formalWorkspacePackages(metadata)
    .filter(
      (pkg) =>
        FORMAL_VERIFICATION_PACKAGES.has(pkg.name) &&
        pkg.id !== undefined &&
        !defaultMembers.has(pkg.id),
    )
    .map(
      (pkg) => `${pkg.name} is missing from formal default workspace members`,
    )
    .sort();
}

async function metadata(): Promise<CargoMetadata> {
  const child = Bun.spawn(["cargo", "metadata", "--format-version", "1"], {
    cwd: process.cwd(),
    stdout: "pipe",
    stderr: "inherit",
  });
  const output = await new Response(child.stdout).text();
  const status = await child.exited;
  if (status !== 0)
    throw new Error(`cargo metadata exited with status ${status}`);
  return JSON.parse(output) as CargoMetadata;
}

async function main(): Promise<void> {
  const cargo = await metadata();
  const violations = [
    ...sharedBoundaryViolations(cargo),
    ...backendFeatureIsolationViolations(cargo),
    ...formalVerificationViolations(cargo),
  ];
  if (violations.length === 0) return;
  throw new Error(
    `The Wabou backend boundaries are invalid:\n${violations
      .map((violation) => `  - ${violation}`)
      .join("\n")}`,
  );
}

if (import.meta.main) await main();
