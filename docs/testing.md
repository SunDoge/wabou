# Behavior testing

Wabou includes a Playwright-style behavior runner for native applications.
Scenarios are TypeScript bundles evaluated in the application's existing
QuickJS runtime. Application trees are still created and reconciled by Solid's
universal renderer; tests do not use a second DOM implementation.

```ts
import { expect, test } from "@wabou/test";

test("toggle the theme", async ({ page }) => {
  await page.getByRole("button", { name: "Light mode" }).click();
  await page.getByRole("button", { name: "Dark mode" }).click();
});
```

`getByRole` reads the same semantic snapshot used by AccessKit. `click()` uses
the semantic node's completed layout bounds, then sends primary pointer down
and up events through native hit-testing and Solid event dispatch. It never
invokes a JSX handler directly.

Run a scenario with the deterministic backend:

```bash
bun run wabou test --app-dir apps/warden-desktop \
  apps/warden-desktop/tests/close-to-tray.test.ts
```

The deterministic backend uses the same Rust window lifecycle state machine as
the winit executor, so Wayland and surface recreation can be tested without a
compositor. It also uses an isolated temporary XDG data directory so persisted
application state cannot make scenarios order-dependent. Pass `--native` for a
real platform smoke test.

Every run writes `report.json` and `trace.json` beneath
`target/wabou-test/<app>/artifacts` by default. Use `--artifacts <dir>` to
select another destination. Deterministic tests do not initialize wgpu, which
keeps them usable in display-less CI. Pass `--failure-screenshot` to opt into a
GPU-rendered `failure.png` when a working wgpu backend is available.
Recorded actions are directly replayable:

```bash
bun run wabou test --app-dir apps/warden-desktop \
  --replay target/wabou-test/warden-desktop/artifacts/trace.json
```

Window behavior is modeled explicitly:

```ts
test("Wayland close-to-tray", async ({ window }) => {
  await window.nativeClose(1, "wayland");
  expect(window.state(1)?.presence).toBe("surface-released");

  await window.show(1);
  expect(window.state(1)?.surfaceGeneration).toBe(2);
});
```

Use `expect.poll(() => value).toBe(expected)` for state that settles across
asynchronous host turns. DevTools remains a diagnostic interface; behavior
assertions belong in scenarios.
