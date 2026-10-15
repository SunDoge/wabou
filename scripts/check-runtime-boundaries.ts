// Check independently: workspace feature unification would hide accidental links.
const forbidden =
  /\b(?:wabou-runtime|wabou-runtime-library|wabou-backend-vello-hybrid|wabou-shell-vello|wabou-render-core|wabou-web|rquickjs|rquickjs-core|rquickjs-sys|vello|vello_hybrid|vello_gpu|wgpu|winit) v/;
for (const name of ["wabou-launch", "gallery-functions", "wabou-cli"]) {
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
const release = Bun.spawnSync(
  [
    "cargo",
    "tree",
    "-p",
    "wabou-runtime-library",
    "--no-default-features",
    "--edges",
    "normal",
  ],
  { stdout: "pipe", stderr: "pipe" },
);
if (release.exitCode !== 0) throw new Error(release.stderr.toString());
const development = release.stdout
  .toString()
  .split("\n")
  .find((line) => /\b(?:wabou-devtools|tungstenite|ureq|notify) v/.test(line));
if (development)
  throw new Error(
    `Release runtime contains development dependencies: ${development}`,
  );
console.log(
  "release runtime: Vite, DevTools and source watcher dependencies are absent",
);

const web = Bun.spawnSync(
  [
    "cargo",
    "tree",
    "-p",
    "wabou-web",
    "--target",
    "wasm32-unknown-unknown",
    "--edges",
    "normal",
  ],
  { stdout: "pipe", stderr: "pipe" },
);
if (web.exitCode !== 0) throw new Error(web.stderr.toString());
const native = web.stdout
  .toString()
  .split("\n")
  .find((line) =>
    /\b(?:wabou-shell-vello|wabou-runtime|rquickjs|rquickjs-core|rquickjs-sys|winit|arboard|rfd|notify-rust|wgpu) v/.test(
      line,
    ),
  );
if (native)
  throw new Error(`Browser preview links native host machinery: ${native}`);
console.log(
  "browser preview: QuickJS, Winit, wgpu and native platform services are absent",
);
