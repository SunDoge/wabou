// Check independently: workspace feature unification would hide accidental links.
const forbidden =
  /\b(?:wabou-runtime|wabou-runtime-library|wabou-backend-vello-hybrid|wabou-shell-vello|blockui-runtime|rquickjs|rquickjs-core|rquickjs-sys|vello|vello_hybrid|wgpu|winit) v/;
for (const name of [
  "wabou-launch",
  "wabou-capability-sdk",
  "gallery-functions",
  "wabou-cli",
]) {
  const result = Bun.spawnSync(
    ["cargo", "tree", "-p", name, "--no-default-features", "--edges", "normal"],
    { stdout: "pipe", stderr: "pipe" },
  );
  if (result.exitCode !== 0) throw new Error(result.stderr.toString());
  const dependency = result.stdout
    .toString()
    .split("\n")
    .find((line) => forbidden.test(line));
  if (dependency) {
    throw new Error(
      `${name} unexpectedly depends on the runtime implementation: ${dependency}`,
    );
  }
  console.log(
    `${name}: runtime implementation is outside the normal dependency graph`,
  );
}
