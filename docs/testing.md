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

Locators also drive captured pointer drags, physical key pairs, committed text,
paste, the full IME preedit/commit lifecycle, and wheel routing through that
same native path:

```ts
const editor = page.getByRole("textbox", { name: "Config" });
await editor.waitFor(); // waits for asynchronously-created semantic content
await editor.type("port: ");
await editor.ime("你好");
await editor.press("a", { control: true });
await editor.dragBy(120, 24);
await editor.wheel(80);

await expect(editor).toHaveValue("port: 你好");
await expect(page.getByRole("status", { name: "Save state" })).toHaveText(
  "Saved",
);
await expect(page.getByRole("button", { name: "Save" })).toBeDisabled();
await expect(page.getByRole("checkbox", { name: "Sync" })).toBeChecked();
await expect(page.getByRole("checkbox", { name: "Partial sync" })).toBeIndeterminate();
await expect(editor).toBeFocused();
```

Locators are bound to logical window 1 by default. Bind explicitly when an
application opens additional native windows; recorded and replayed actions
preserve the same window id:

```ts
const child = page.forWindow(2);
await child.getByRole("button", { name: "Close" }).click();
```

Locator state assertions read the completed native semantic snapshot, not
application signals. This makes them useful for catching failures between
Solid reconciliation, Wabou's protocol, layout, and accessibility projection.
They automatically cross a native frame barrier and retry until the expected
state is visible. Use `page.waitForIdle()` directly only when a test must wait
for native layout/semantics without making an assertion; it is not implemented
as an arbitrary sleep or a fixed number of JavaScript animation frames.

Run a scenario with the deterministic backend:

```bash
bun run wabou test /path/to/app/tests/window-lifecycle.test.ts \
  --app /path/to/app
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
bun run wabou test --replay target/wabou-test/app/artifacts/trace.json \
  --app /path/to/app
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
