import { relative, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");

export async function discoverBehaviorApps(
  workspaceRoot = root,
): Promise<string[]> {
  const applications = new Set<string>();
  const glob = new Bun.Glob("apps/*/tests/**/*.behavior.ts");

  for await (const source of glob.scan({
    cwd: workspaceRoot,
    onlyFiles: true,
  })) {
    const [apps, application] = source.split(/[\\/]/u);
    if (apps === "apps" && application) {
      applications.add(`${apps}/${application}`);
    }
  }

  return [...applications].sort();
}

async function main(): Promise<void> {
  const applications = await discoverBehaviorApps();
  if (applications.length === 0) {
    throw new Error("no apps/*/tests/**/*.behavior.ts suites were discovered");
  }

  if (process.argv.includes("--list")) {
    for (const application of applications) console.log(application);
    return;
  }

  for (const application of applications) {
    console.log(
      `[behavior] testing ${relative(root, resolve(root, application))}`,
    );
    const child = Bun.spawn(
      ["cargo", "run", "-p", "wabou-cli", "--", "test", application],
      {
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
      },
    );
    const exitCode = await child.exited;
    if (exitCode !== 0) process.exit(exitCode);
  }

  console.log(
    `verified native behavior for ${applications.length} applications`,
  );
}

if (import.meta.main) await main();
