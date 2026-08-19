import { expect, test } from "bun:test";
import { EFFECT_ABI_VERSION } from "../generated/effect-abi";

const submissions: Array<[number, number, string]> = [];
Object.assign(globalThis, {
  __wabou_effect_abi: EFFECT_ABI_VERSION,
  __wabou_effect_submit: (
    capability: number,
    method: number,
    payload: string,
  ) => {
    submissions.push([capability, method, payload]);
    return submissions.length;
  },
});

const { appDirs, resolveAppDirectories } = await import("./app-dirs");

test("app directories resolve once and expose named roots", async () => {
  const all = resolveAppDirectories();
  const localData = appDirs.localData();

  expect(submissions).toEqual([[4, 1, "null"]]);
  __wabou_effect_complete(
    1,
    4,
    1,
    0,
    JSON.stringify({
      configDir: "/app/config",
      dataDir: "/app/data",
      localDataDir: "/app/local",
      cacheDir: "/app/cache",
      logDir: "/app/log",
      resourceDir: "/app/resources",
      tempDir: "/tmp/app",
    }),
  );

  expect((await all).resourceDir).toBe("/app/resources");
  expect(await localData).toBe("/app/local");
  expect(await appDirs.cache()).toBe("/app/cache");
  expect(submissions).toHaveLength(1);
});
