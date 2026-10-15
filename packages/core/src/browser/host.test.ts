import { expect, test } from "bun:test";
import { fileURLToPath } from "node:url";

test("browser host cleans up and can restart without stale protocol atoms", async () => {
  const child = Bun.spawn(
    [
      process.execPath,
      "--conditions=browser",
      "--conditions=wabou-source",
      fileURLToPath(new URL("./host.fixture.ts", import.meta.url)),
    ],
    { stdout: "pipe", stderr: "pipe" },
  );
  const [code, output, error] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  expect(code, error).toBe(0);
  expect(output).toContain("entry/font rollback passed");
});
