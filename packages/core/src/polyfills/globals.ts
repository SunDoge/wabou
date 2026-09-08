/** Install a writable runtime global without replacing a host implementation. */
export function installMissingGlobal(name: string, value: unknown): boolean {
  if (name in globalThis) return false;
  Object.defineProperty(globalThis, name, {
    configurable: true,
    writable: true,
    value,
  });
  return true;
}

/** Install a related set of runtime globals with the same preservation rule. */
export function installMissingGlobals(
  values: Readonly<Record<string, unknown>>,
): void {
  for (const [name, value] of Object.entries(values)) {
    installMissingGlobal(name, value);
  }
}
