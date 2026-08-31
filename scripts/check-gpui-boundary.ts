const FORMAL_PACKAGES = new Set([
  "wabou",
  "wabou-cli",
  "wabou-runtime",
  "wabou-shell",
]);

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
  name: string;
}

interface CargoMetadata {
  packages: CargoPackage[];
}

export function gpuiBoundaryViolations(metadata: CargoMetadata): string[] {
  const violations: string[] = [];
  for (const pkg of metadata.packages) {
    if (!FORMAL_PACKAGES.has(pkg.name)) continue;
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
  const violations = gpuiBoundaryViolations(await metadata());
  if (violations.length === 0) return;
  throw new Error(
    `Formal GPUI packages depend on retired renderer crates in normal, build, or test code:\n${violations
      .map((violation) => `  - ${violation}`)
      .join("\n")}`,
  );
}

if (import.meta.main) await main();
