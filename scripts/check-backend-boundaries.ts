const BACKEND_LIBRARY_DEPENDENCIES = new Set([
  "anyrender",
  "anyrender_skia",
  "anyrender_svg",
  "anyrender_vello",
  "anyrender_vello_hybrid",
  "anyrender-skia",
  "anyrender-svg",
  "anyrender-vello",
  "anyrender-vello-hybrid",
  "gpui-ce",
  "gpui-pre",
  "gpui_ce_components_base",
  "gpui_ce_platform",
  "vello",
  "vello_hybrid",
  "winit",
]);

const SHARED_PACKAGES = new Set([
  "wabou-accessibility",
  "wabou-bindgen",
  "wabou-database",
  "wabou-devtools",
  "wabou-host-api",
  "wabou-protocol",
  "wabou-shell-api",
  "wabou-style",
  "wabou-terminal-core",
]);

const GPUI_BACKEND_PACKAGES = new Set([
  "wabou-runtime",
  "wabou-shell",
  "wabou-terminal",
  "wabou-tray",
]);

const HYBRID_BACKEND_PACKAGES = new Set([
  "wabou-legacy-accessibility",
  "wabou-legacy-runtime",
  "wabou-legacy-shell",
  "wabou-legacy-terminal",
  "wabou-legacy-widgets",
  "wabou-vello-hybrid-svg",
]);

// These are the two known extraction seams described in rendering-roadmap.md.
// Freeze them so the Hybrid backend cannot acquire more GPUI dependencies while
// RuntimeSession and the remaining backend-neutral contracts move out.
const TRANSITIONAL_CROSS_BACKEND_EDGES = new Set([
  "wabou-legacy-runtime -> wabou-runtime",
  "wabou-legacy-runtime -> wabou-shell",
]);

interface CargoDependency {
  kind: "build" | "dev" | null;
  name: string;
  rename: string | null;
}

interface CargoPackage {
  dependencies: CargoDependency[];
  id?: string;
  name: string;
  publish?: string[] | null;
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
      !pkg.name.startsWith("wabou-legacy-") &&
      (members.size === 0 || (pkg.id !== undefined && members.has(pkg.id))),
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

function isBackendDependency(name: string): boolean {
  return (
    BACKEND_LIBRARY_DEPENDENCIES.has(name) ||
    GPUI_BACKEND_PACKAGES.has(name) ||
    HYBRID_BACKEND_PACKAGES.has(name)
  );
}

/** Backend-neutral crates must remain usable without either platform stack. */
export function sharedBoundaryViolations(metadata: CargoMetadata): string[] {
  const violations: string[] = [];
  for (const pkg of metadata.packages) {
    if (!SHARED_PACKAGES.has(pkg.name)) continue;
    for (const dependency of pkg.dependencies) {
      if (isBackendDependency(dependency.name)) {
        violations.push(dependencyLabel(pkg, dependency));
      }
    }
  }
  return violations.sort();
}

/** Each backend may consume shared crates, but must not grow a dependency on the other. */
export function crossBackendViolations(metadata: CargoMetadata): string[] {
  const violations: string[] = [];
  for (const pkg of metadata.packages) {
    const gpui = GPUI_BACKEND_PACKAGES.has(pkg.name);
    const hybrid = HYBRID_BACKEND_PACKAGES.has(pkg.name);
    if (!gpui && !hybrid) continue;
    for (const dependency of pkg.dependencies) {
      const crosses =
        (gpui && HYBRID_BACKEND_PACKAGES.has(dependency.name)) ||
        (hybrid && GPUI_BACKEND_PACKAGES.has(dependency.name));
      if (!crosses) continue;
      const edge = `${pkg.name} -> ${dependency.name}`;
      if (TRANSITIONAL_CROSS_BACKEND_EDGES.has(edge)) continue;
      violations.push(dependencyLabel(pkg, dependency));
    }
  }
  return violations.sort();
}

export function transitionalPackagingViolations(
  metadata: CargoMetadata,
): string[] {
  const defaultMembers = new Set(metadata.workspace_default_members ?? []);
  const violations: string[] = [];
  for (const pkg of metadata.packages) {
    if (!pkg.name.startsWith("wabou-legacy-")) continue;
    // Cargo represents `publish = false` as an empty registry list.
    if (pkg.publish === undefined || pkg.publish === null) {
      violations.push(`${pkg.name} is publishable`);
    }
    if (pkg.id !== undefined && defaultMembers.has(pkg.id)) {
      violations.push(`${pkg.name} is a default workspace member`);
    }
  }
  return violations.sort();
}

export function formalVerificationViolations(
  metadata: CargoMetadata,
): string[] {
  const defaultMembers = new Set(metadata.workspace_default_members ?? []);
  return formalWorkspacePackages(metadata)
    .filter((pkg) => pkg.id !== undefined && !defaultMembers.has(pkg.id))
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
    ...crossBackendViolations(cargo),
    ...transitionalPackagingViolations(cargo),
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
