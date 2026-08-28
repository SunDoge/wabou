import { readdir } from "node:fs/promises";
import { join } from "node:path";

const root = new URL("..", import.meta.url).pathname;
const version = process.argv[2];
const prereleasePattern = /^\d+\.\d+\.\d+-[0-9A-Za-z.-]+$/;

if (!version || !prereleasePattern.test(version)) {
  throw new Error(
    "usage: bun run release:prepare <prerelease-version> (for example 0.1.0-alpha.3)",
  );
}

const tag = `v${version}`;

async function command(args: string[], options: { capture?: boolean } = {}) {
  const child = Bun.spawn(args, {
    cwd: root,
    stdin: "inherit",
    stdout: options.capture ? "pipe" : "inherit",
    stderr: options.capture ? "pipe" : "inherit",
  });
  const [status, stdout, stderr] = await Promise.all([
    child.exited,
    options.capture ? new Response(child.stdout).text() : "",
    options.capture ? new Response(child.stderr).text() : "",
  ]);
  return { status, stdout: stdout.trim(), stderr: stderr.trim() };
}

async function requireCleanWorktree() {
  const result = await command(["git", "status", "--porcelain"], {
    capture: true,
  });
  if (result.status !== 0)
    throw new Error(result.stderr || "git status failed");
  if (result.stdout) {
    throw new Error(
      "release preparation requires a clean worktree; commit or stash existing changes first",
    );
  }
}

async function requireUnusedTag() {
  const local = await command(["git", "tag", "--list", tag], {
    capture: true,
  });
  if (local.status !== 0) throw new Error(local.stderr || "git tag failed");
  if (local.stdout) throw new Error(`local tag ${tag} already exists`);

  const remote = await command(
    ["git", "ls-remote", "--exit-code", "--tags", "origin", `refs/tags/${tag}`],
    { capture: true },
  );
  if (remote.status === 0) throw new Error(`remote tag ${tag} already exists`);
  // git ls-remote returns 2 when the remote is reachable but has no match.
  if (remote.status !== 2) {
    throw new Error(
      `could not verify remote tag ${tag}: ${remote.stderr || `exit ${remote.status}`}`,
    );
  }
}

function releaseRootChangelog(source: string): string {
  const unreleased = "## Unreleased";
  const start = source.indexOf(unreleased);
  if (start < 0) throw new Error("CHANGELOG.md has no Unreleased section");
  if (source.includes(`## ${version} -`)) {
    throw new Error(`CHANGELOG.md already contains ${version}`);
  }
  const bodyStart = start + unreleased.length;
  const nextHeading = source.indexOf("\n## ", bodyStart);
  if (nextHeading < 0) throw new Error("CHANGELOG.md has no previous release");
  const body = source.slice(bodyStart, nextHeading).trim();
  if (!body) throw new Error("CHANGELOG.md Unreleased section is empty");
  const date = new Date().toISOString().slice(0, 10);
  return `${source.slice(0, bodyStart)}\n\n## ${version} - ${date}\n\n${body}\n${source.slice(nextHeading)}`;
}

async function updateText(path: string, update: (source: string) => string) {
  const source = await Bun.file(path).text();
  const updated = update(source);
  if (updated === source)
    throw new Error(`${path} did not contain the expected release metadata`);
  await Bun.write(path, updated);
}

await requireCleanWorktree();
await requireUnusedTag();

const packageRoot = join(root, "packages");
const packageDirectories = (await readdir(packageRoot, { withFileTypes: true }))
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name);
const packageManifests = (
  await Promise.all(
    packageDirectories.map(async (directory) => {
      const path = join(packageRoot, directory, "package.json");
      return (await Bun.file(path).exists()) ? path : undefined;
    }),
  )
).filter((path): path is string => path !== undefined);

const oldVersions = new Set<string>();
for (const path of packageManifests) {
  const manifest = await Bun.file(path).json();
  if (manifest.private !== true) oldVersions.add(manifest.version);
}
if (oldVersions.size !== 1) {
  throw new Error(
    `public packages do not share one version: ${[...oldVersions]}`,
  );
}
const oldVersion = [...oldVersions][0];
if (oldVersion === version)
  throw new Error(`workspace is already at ${version}`);

const oldPackageChangelogs = new Map<string, string>();
for (const path of packageManifests) {
  const manifest = await Bun.file(path).json();
  if (manifest.private === true) continue;
  const changelogPath = join(path, "..", "CHANGELOG.md");
  if (await Bun.file(changelogPath).exists()) {
    oldPackageChangelogs.set(
      changelogPath,
      await Bun.file(changelogPath).text(),
    );
  }
}

// Changesets owns package release notes and consumes pending changesets. The
// explicit pass below then corrects its computed prerelease to the requested
// version, which is necessary when an earlier Git tag was created by mistake.
const changeset = await command(["bun", "x", "changeset", "version"]);
if (changeset.status !== 0) throw new Error("changeset version failed");

for (const path of packageManifests) {
  const manifest = await Bun.file(path).json();
  if (manifest.private === true) continue;
  const computedVersion = manifest.version as string;
  manifest.version = version;
  await Bun.write(path, `${JSON.stringify(manifest, null, 2)}\n`);

  const changelogPath = join(path, "..", "CHANGELOG.md");
  if (!(await Bun.file(changelogPath).exists())) continue;
  const changelog = await Bun.file(changelogPath).text();
  if (changelog.includes(`## ${version}\n`)) continue;
  const oldChangelog = oldPackageChangelogs.get(changelogPath);
  if (
    computedVersion !== oldVersion &&
    oldChangelog &&
    changelog.endsWith(oldChangelog)
  ) {
    const releaseNotes = changelog
      .slice(0, -oldChangelog.length)
      .replaceAll(computedVersion, version);
    await Bun.write(changelogPath, `${releaseNotes}${oldChangelog}`);
  } else {
    const titleEnd = changelog.indexOf("\n");
    await Bun.write(
      changelogPath,
      `${changelog.slice(0, titleEnd + 1)}\n## ${version}\n\n### Patch Changes\n\n- Prepare the ${version} developer preview.\n${changelog.slice(titleEnd + 1)}`,
    );
  }
}

await updateText(join(root, "Cargo.toml"), (source) =>
  source.replace(
    /(^\[workspace\.package]\nversion = ")[^"]+("$)/m,
    `$1${version}$2`,
  ),
);
await updateText(join(root, "CHANGELOG.md"), releaseRootChangelog);

for (const lockfile of ["Cargo.lock", "bun.lock"]) {
  await updateText(join(root, lockfile), (source) =>
    source.replaceAll(oldVersion, version),
  );
}

const revisionFiles = [
  "README.md",
  "docs/cli.md",
  "crates/wabou/README.md",
  "crates/wabou-cli/src/main.rs",
];
for (const relativePath of revisionFiles) {
  await updateText(join(root, relativePath), (source) =>
    source.replaceAll(`v${oldVersion}`, tag),
  );
}

for (const args of [
  ["bun", "install"],
  ["bun", "run", "scripts/check-package-boundaries.ts"],
]) {
  const result = await command(args);
  if (result.status !== 0) throw new Error(`${args.join(" ")} failed`);
}

const metadata = await command(
  ["cargo", "metadata", "--format-version", "1", "--no-deps"],
  { capture: true },
);
if (metadata.status !== 0)
  throw new Error(metadata.stderr || "cargo metadata failed");

console.log(
  `\nPrepared ${version}. Review and commit the changes before creating ${tag}.`,
);
console.log(`After CI passes, create the tag with:`);
console.log(`  git tag -a ${tag} -m "Wabou ${tag}"`);
console.log(`The script never commits, tags, publishes, or pushes.`);
