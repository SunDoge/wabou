import { expect, test } from "bun:test";
import { installMissingGlobal, installMissingGlobals } from "./globals";

test("installs writable globals without replacing host implementations", () => {
  const firstName = `__wabou_test_global_${Date.now()}_first`;
  const secondName = `__wabou_test_global_${Date.now()}_second`;
  const hostValue = { owner: "host" };

  Object.defineProperty(globalThis, firstName, {
    configurable: true,
    writable: true,
    value: hostValue,
  });

  try {
    expect(installMissingGlobal(firstName, { owner: "polyfill" })).toBe(false);
    installMissingGlobals({ [secondName]: 1 });
    expect((globalThis as Record<string, unknown>)[firstName]).toBe(hostValue);
    expect((globalThis as Record<string, unknown>)[secondName]).toBe(1);

    (globalThis as Record<string, unknown>)[secondName] = 2;
    expect((globalThis as Record<string, unknown>)[secondName]).toBe(2);
    expect(
      Object.getOwnPropertyDescriptor(globalThis, secondName),
    ).toMatchObject({
      configurable: true,
      writable: true,
    });
  } finally {
    Reflect.deleteProperty(globalThis, firstName);
    Reflect.deleteProperty(globalThis, secondName);
  }
});
