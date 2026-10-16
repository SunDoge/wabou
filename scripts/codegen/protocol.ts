// Cross-language host and renderer protocol artifacts.
await import("./protocol/host-abi.ts");
await import("./protocol/effect-abi.ts");

const protocolBindings = Bun.spawn(
  [
    "cargo",
    "run",
    "-p",
    "wabou-protocol",
    "--example",
    "protocol-bindings",
    "--",
    "write",
  ],
  { cwd: `${import.meta.dir}/../..`, stdout: "inherit", stderr: "inherit" },
);
if ((await protocolBindings.exited) !== 0) {
  throw new Error("Rust-owned protocol binding generation failed");
}

export {};
