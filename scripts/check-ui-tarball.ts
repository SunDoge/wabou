import { access, mkdir, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const root = new URL("..", import.meta.url).pathname;
const temporary = await mkdtemp(join(tmpdir(), "wabou-ui-consumer-"));
const obsoleteUiPackages = [
  "@wabou/animation",
  "@wabou/components",
  "@wabou/primitives",
  "@wabou/router",
];

async function run(command: string[], cwd = root): Promise<void> {
  const child = Bun.spawn(command, {
    cwd,
    stdin: "ignore",
    stdout: "inherit",
    stderr: "inherit",
  });
  const status = await child.exited;
  if (status !== 0) {
    throw new Error(`${command.join(" ")} exited with status ${status}`);
  }
}

const workspaceManifest = await Bun.file(join(root, "package.json")).json();
const coreManifest = await Bun.file(
  join(root, "packages/core/package.json"),
).json();
const uiManifest = await Bun.file(
  join(root, "packages/ui/package.json"),
).json();
const viteManifest = await Bun.file(
  join(root, "packages/vite/package.json"),
).json();
await run(
  ["bun", "pm", "pack", "--destination", temporary],
  join(root, "packages/core"),
);
await run(
  ["bun", "pm", "pack", "--destination", temporary],
  join(root, "packages/ui"),
);
await run(
  ["bun", "pm", "pack", "--destination", temporary],
  join(root, "packages/vite"),
);

const coreTarball = join(temporary, `wabou-core-${coreManifest.version}.tgz`);
const uiTarball = join(temporary, `wabou-ui-${uiManifest.version}.tgz`);
const viteTarball = join(temporary, `wabou-vite-${viteManifest.version}.tgz`);
await writeFile(
  join(temporary, "package.json"),
  `${JSON.stringify(
    {
      name: "wabou-ui-tarball-consumer",
      private: true,
      type: "module",
      dependencies: {
        "@solidjs/web": workspaceManifest.devDependencies["@solidjs/web"],
        "@wabou/ui": `file:${uiTarball}`,
        "solid-js": coreManifest.dependencies["solid-js"],
      },
      devDependencies: {
        "@wabou/vite": `file:${viteTarball}`,
        typescript: workspaceManifest.devDependencies.typescript,
        vite: workspaceManifest.devDependencies.vite,
      },
      overrides: { "@wabou/core": `file:${coreTarball}` },
    },
    null,
    2,
  )}\n`,
);
await writeFile(
  join(temporary, "tsconfig.json"),
  `${JSON.stringify(
    {
      compilerOptions: {
        strict: true,
        noEmit: true,
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "Bundler",
        jsx: "preserve",
        jsxImportSource: "@wabou/ui",
        skipLibCheck: true,
      },
      include: ["ui/**/*.tsx"],
    },
    null,
    2,
  )}\n`,
);
await writeFile(
  join(temporary, "wabou.config.ts"),
  `import { defineConfig } from "@wabou/vite";

export default defineConfig({ outDir: "dist" });
`,
);
await mkdir(join(temporary, "ui"));
await writeFile(
  join(temporary, "ui/index.tsx"),
  `import { Button, createDataRouter, createMemoryHistory, View } from "@wabou/ui";
import { Text } from "@wabou/ui/primitives";

const history = createMemoryHistory();
void createDataRouter;
void history;
export const Example = () => <View><Button>Run</Button><Text>Ready</Text></View>;
`,
);

await run(["bun", "install"], temporary);
await run(["bun", "x", "tsc", "--noEmit"], temporary);
await run(
  ["bun", "x", "vite", "--config", "wabou.config.ts", "build"],
  temporary,
);

for (const packageName of obsoleteUiPackages) {
  const internal = packageName.slice("@wabou/".length);
  const installed = await access(
    join(temporary, "node_modules/@wabou", internal),
  ).then(
    () => true,
    () => false,
  );
  if (installed) {
    throw new Error(
      `isolated @wabou/ui consumer installed private ${internal}`,
    );
  }
}

const installedManifest = JSON.parse(
  await readFile(
    join(temporary, "node_modules/@wabou/ui/package.json"),
    "utf8",
  ),
);
for (const packageName of obsoleteUiPackages) {
  if (installedManifest.dependencies?.[packageName]) {
    throw new Error(`published @wabou/ui still depends on ${packageName}`);
  }
}

console.log("verified isolated @wabou/ui tarball consumer");
