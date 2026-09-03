const RETIRED_DIRECT_DEPENDENCIES = new Set([
  "anyrender",
  "anyrender-skia",
  "anyrender-svg",
  "anyrender-vello",
  "vello",
  "winit",
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

export function gpuiBoundaryViolations(metadata: CargoMetadata): string[] {
  const violations: string[] = [];
  for (const pkg of formalWorkspacePackages(metadata)) {
    for (const dependency of pkg.dependencies) {
      const visibleName = dependency.rename ?? dependency.name;
      if (
        dependency.name.startsWith("wabou-legacy-") ||
        RETIRED_DIRECT_DEPENDENCIES.has(dependency.name)
      ) {
        const kind = dependency.kind ?? "normal";
        violations.push(
          `${pkg.name} -> ${visibleName} (${dependency.name}, ${kind})`,
        );
      }
    }
  }
  return violations.sort();
}

export function canonicalDependencyViolations(
  metadata: CargoMetadata,
): string[] {
  const violations: string[] = [];
  for (const pkg of formalWorkspacePackages(metadata)) {
    for (const dependency of pkg.dependencies) {
      if (dependency.name === "wabou-shell" && dependency.rename !== null) {
        violations.push(
          `${pkg.name} renames wabou-shell to ${dependency.rename}`,
        );
      }
    }
  }
  return violations.sort();
}

export function legacyIsolationViolations(metadata: CargoMetadata): string[] {
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
    ...gpuiBoundaryViolations(cargo),
    ...canonicalDependencyViolations(cargo),
    ...legacyIsolationViolations(cargo),
    ...formalVerificationViolations(cargo),
  ];
  if (violations.length === 0) return;
  throw new Error(
    `The formal GPUI crate boundary is invalid:\n${violations
      .map((violation) => `  - ${violation}`)
      .join("\n")}`,
  );
}

if (import.meta.main) await main();
