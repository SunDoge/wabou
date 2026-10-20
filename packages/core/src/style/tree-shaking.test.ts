import { expect, test } from "bun:test";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

test("typed style values do not load the utility conflict resolver", async () => {
  const directory = await mkdtemp(join(tmpdir(), "wabou-style-bundle-"));
  try {
    const entry = join(directory, "entry.ts");
    const source = fileURLToPath(new URL("./index.ts", import.meta.url));
    await Bun.write(
      entry,
      `export { px, rgba } from ${JSON.stringify(source.replaceAll("\\", "/"))};`,
    );
    const build = await Bun.build({ entrypoints: [entry], target: "browser" });
    expect(build.success).toBe(true);
    const output = await build.outputs[0].text();
    expect(output).not.toContain("propertyGroups");
    expect(output).not.toContain("staticUtilities");
    expect(output).not.toContain("dynamicRules");
    const module = await import(
      `data:text/javascript;base64,${Buffer.from(output).toString("base64")}`
    );
    expect(module.px(12)).toMatchObject({ kind: 1, value: 12 });
    expect(module.rgba(0xffffffff)).toMatchObject({
      kind: 5,
      value: 0xffffffff,
    });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
