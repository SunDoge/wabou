import { basename, resolve } from "node:path";

interface CargoTarget {
  name: string;
  kind: string[];
  src_path: string;
  "required-features"?: string[];
}

interface CargoPackage {
  name: string;
  manifest_path: string;
  targets: CargoTarget[];
}

interface CargoMetadata {
  packages: CargoPackage[];
  workspace_root: string;
}

const root = resolve(import.meta.dir, "..");
const metadataChild = Bun.spawn(
  ["cargo", "metadata", "--format-version", "1", "--no-deps"],
  { cwd: root, stdout: "pipe", stderr: "inherit" },
);
const metadataText = await new Response(metadataChild.stdout).text();
if ((await metadataChild.exited) !== 0)
  throw new Error("failed to read Cargo metadata");
const metadata = JSON.parse(metadataText) as CargoMetadata;

const generators = metadata.packages
  .filter((pkg) => pkg.manifest_path.startsWith(`${metadata.workspace_root}/`))
  .flatMap((pkg) =>
    pkg.targets
      .filter(
        (target) =>
          target.kind.includes("example") &&
          /(?:bindgen|bindings)\.rs$/.test(basename(target.src_path)),
      )
      .map((target) => ({ pkg: pkg.name, target })),
  )
  .sort((left, right) =>
    `${left.pkg}:${left.target.name}`.localeCompare(
      `${right.pkg}:${right.target.name}`,
    ),
  );

if (generators.length === 0)
  throw new Error("no Rust-owned TypeScript binding generators discovered");

for (const { pkg, target } of generators) {
  const args = ["cargo", "run", "-p", pkg];
  const requiredFeatures = target["required-features"] ?? [];
  if (requiredFeatures.length > 0)
    args.push("--features", requiredFeatures.join(","));
  args.push("--example", target.name, "--", "check");
  console.log(`[bindings] checking ${pkg}:${target.name}`);
  const child = Bun.spawn(args, {
    cwd: root,
    stdout: "inherit",
    stderr: "inherit",
  });
  if ((await child.exited) !== 0)
    throw new Error(`binding generator failed: ${pkg}:${target.name}`);
}

console.log(`verified ${generators.length} Rust-owned TypeScript bindings`);
