# Testing

## Verification tiers

`bun run verify` is the complete local and pre-release suite. It checks every
Rust target and feature combination, JavaScript packages, generated bindings,
native behavior, layout contracts, and authored captures.

GitHub Actions intentionally uses the bounded `bun run verify:ci:rust` merge
gate instead. It runs formatting, Clippy for libraries, binaries, and tests,
then workspace library tests. Bindings, native behavior, and layout contracts
remain separate CI steps. Expensive all-target checks, replay variants,
standalone scaffold builds, captures, HiDPI renders, and performance sampling
belong to local pre-release verification rather than every pushed commit.

## Solid-to-GPUI architecture contracts

The GPUI backend has two non-negotiable retained-runtime contracts. Test them
below component and pixel layers so visual success cannot hide duplicated trees
or excessive native rebuilding.

### Fine-grained projection

A Solid write must produce one completed protocol frame, merge repeated writes
to the same node, and advance only the nearest explicit GPUI projection
boundary. The contract is split into deterministic layers:

1. renderer tests execute real Solid and assert the exact mutation frame
   emitted by a signal;
2. `wabou-shell` tests feed that operation into the retained projection and
   assert dirty-kind coalescing plus independent boundary revision clocks;
3. headless GPUI tests assert that an unrelated root notification or an
   animation-only frame does not rematerialize cached projection boundaries.

Do not replace these assertions with FPS thresholds. Timing is machine-specific;
mutation count, changed boundary identity, and materialization count are the
stable architecture contracts.

### HMR lifecycle

HMR must preserve one logical application tree at all times:

1. a normal Solid Refresh replaces the component while retaining parent state
   and parent mount identity;
2. entry-module and generated Style IR updates are classified as side-effect
   updates instead of being forced through a component accept boundary;
3. a remount emits retirement and replacement operations in the same host
   frame, leaving the previous root detached;
4. dropping a generation clears native hover, press, click, and focus state;
5. a native smoke cycle may additionally assert baseline node count → changed
   route node count → baseline node count and run `wabou inspect validate`.

The first four checks belong in the normal test suite and require no Vite
server or screenshot. Keep the fifth as a focused integration check for changes
to the Vite client, module loader, or native reload pump.

## Authored captures and themes

Use authored captures only when component behavior and layout contracts already
pass but paint, typography, elevation, or theme contrast still needs visual
evidence. `wabou render` accepts an explicit system color scheme so a headless
run does not silently validate only the light theme:

```bash
bun run wabou render apps/gallery \
  --scenario apps/gallery/captures/alert.ts \
  --color-scheme dark \
  --out /tmp/gallery-dark.png \
  --snapshot /tmp/gallery-dark.json
```

Application capture suites can set the same contract in
`captures/config.json`. The default is `light`; use per-scenario overrides when
the same suite deliberately covers both schemes:

```json
{
  "defaults": {
    "width": 1200,
    "height": 850,
    "scaleFactor": 1,
    "colorScheme": "light"
  },
  "overrides": {
    "settings-dark.ts": {
      "colorScheme": "dark"
    }
  }
}
```

Capture discovery only treats `captures/**/*.behavior.ts` as scenarios. When
an application needs a deterministic child process, local service, or other
host dependency, add `captures/setup.ts` with a default async function that
returns environment overrides:

```ts
export default async function prepareCaptures() {
  // Build or start the deterministic fixture here.
  return { MY_APP_FIXTURE: "/absolute/path/to/fixture" };
}
```

The setup runs once per application before its first authored capture and is
skipped by `--check-existing`. This keeps the generic capture command faithful
to the application's real host boundary without silently contacting live
services.

`wabou layout` also accepts `--color-scheme light|dark` for theme-dependent
layout fixtures. Prefer one structural fixture when geometry is theme-neutral;
author both variants only when typography or token differences can change the
layout.

`defineLayoutFixtures` follows the native `WindowMetrics.colorScheme` by
default and selects the matching `light` or `dark` compiled theme. Applications
whose authored theme names differ can map them explicitly:

```tsx
defineLayoutFixtures(fixtures, {
  colorTheme: (scheme) => (scheme === "dark" ? "midnight" : "daylight"),
});
```

Pass `{ colorTheme: false }` only when the fixture wrapper deliberately owns a
fixed `ColorThemeProvider`. Node-side tests can set `colorScheme` on
`renderAppLayout` or `renderLayoutFixtures`; this is forwarded to the same
native CLI contract rather than emulating theme resolution in the test runner.

## Component unit tests

Component anatomy is governed by the
[component composition contracts](component-contracts.md). Use the structural
assertions from `@wabou/test/component` to reject duplicate surface ownership,
unexpected focus owners, and overlays mounted in the content plane before a
native behavior test is necessary.

Most component behavior should be tested without starting a Rust host, native
window, layout engine, or GPU. `@wabou/test/component` mounts TSX through the
real Wabou Solid renderer and records the authored host tree, so a component
test can use role-based locators and dispatch the same JavaScript event path as
the application:

```tsx
import { renderComponent } from "@wabou/test/component";
import { Button, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

function Counter() {
  const [count, setCount] = createSignal(0);
  return (
    <View>
      <Button aria-label="Increment" onClick={() => setCount(count() + 1)}>
        Increment
      </Button>
      <Text role="status" aria-label={`Count ${count()}`}>
        {String(count())}
      </Text>
    </View>
  );
}

test("increments", () => {
  const screen = renderComponent(() => <Counter />);
  screen.getByRole("button", { name: "Increment" }).click();
  expect(screen.getByRole("status", { name: "Count 1" }).text).toBe("1");
});
```

Use the shared transform in `vitest.config.ts`:

```ts
import { defineWabouTestConfig } from "@wabou/vite/test";

export default defineWabouTestConfig();
```

The configuration also cleans up the mounted component after each test and
deduplicates Solid so application components and the test renderer share one
reactive graph. Locators are strict: duplicate role/name matches fail unless an
explicit zero-based `index` is supplied. Prefer `getAllByRole` for intentional
collections and query within a structural locator when names repeat across
independent composed components:

```ts
const forms = screen.getAllByRole("group");
const profile = screen.getByRole("group", { name: "Profile form" });
profile.getByRole("button", { name: "Save" }).click();
expect(profile.queryAllByRole("button")).toHaveLength(2);
```

Scoped queries search descendants rather than the scope node itself and follow
reactive insertions and removals. Querying through a locator whose root has
been removed fails explicitly instead of inspecting a stale detached tree.
`click()` and `press(key)` dispatch through Wabou's real JavaScript event path
and flush Solid synchronously, so the next assertion observes the completed
component update. Common semantic states are exposed directly as live
locator properties (`disabled`, `readOnly`, `checked`, `selected`, `expanded`,
`pressed`, `current`, and `orientation`), so component tests assert the same
contract consumed by native accessibility projection instead of stringly
typed `aria-*` attributes:

```ts
const updates = screen.getByRole("checkbox", { name: "Updates" });
updates.click();
expect(updates.checked).toBe(true);
```

Pointer-driven components can distinguish ordinary movement from a captured
drag: `movePointer()` emits a native move with no pressed buttons, while
`pointerMove()` preserves the primary-button state between `pointerDown()` and
`pointerUp()`. This keeps hover selection and drag behavior on their real event
paths.

The same states can filter role queries, including scoped and list queries.
Filters remain live across component updates, so tests can locate the selected,
expanded, current, or focused item without inspecting raw attributes:

```ts
expect(screen.getByRole("tab", { selected: true }).name).toBe("General");
expect(screen.queryByRole("button", { focused: true })).toBeNull();
```

Value-bearing locators likewise expose `value`, `numericValue`,
`minNumericValue`, `maxNumericValue`, and `valueText`. These remain live after
component events and parse numeric semantics once in the harness:

```ts
const slider = screen.getByRole("slider", { name: "Volume" });
slider.press("ArrowRight");
expect(slider.numericValue).toBe(45);
```

`input(value)` first focuses the editor and then dispatches Wabou's authored
input event. It rejects disabled and read-only editors. Native editing details
such as selection, clipboard paste, and IME composition remain behavior-test
responsibilities because the Rust editor owns them.

Components that subscribe to native measurement can still mount in their
explicit unmeasured state without requiring a window.

For delayed overlays, debouncing, or animation bookkeeping, let the harness
own Vitest's fake clock. Awaiting `advanceTime` advances timers and Wabou's
native-frame queue, committing the resulting Solid and protocol work at
deterministic 16ms frame boundaries while allowing animation microtasks to
settle; disposing the screen restores real time:

```ts
const screen = renderComponent(
  () => (
    <Tooltip
      openDelay={400}
      trigger={(trigger) => <Button {...trigger}>Help</Button>}
    >
      Help text
    </Tooltip>
  ),
  { clock: "fake" },
);
await screen.advanceTime(400);
expect(screen.getByRole("tooltip")).not.toBeNull();
```

Behavior assertions should use roles and semantic state. For the narrower case
where a component contract includes emitted paint state, a locator exposes its
direct authored `children`, `style(name)`, and `transform`. This permits narrow
visual-contract assertions without adding test-only roles or attributes to
production components:

```ts
const control = screen.getByRole("switch", { name: "Sync" });
const thumb = control.children[0];
await screen.advanceTime(90);
expect(thumb.transform?.[4]).toBeGreaterThan(0);
```

`style(name)` observes string and typed inline values emitted through Wabou's
actual protocol writer and remains live across reactive updates. It does not
pretend to be computed layout or pixel output; keep those checks in native
layout and screenshot tests.

Promise-backed capabilities and async event handlers can be observed with the
harness-owned retry loop. `waitFor` commits Solid and protocol work before each
assertion, reports the last locator/assertion failure on timeout, and does not
implicitly advance a fake clock:

```ts
screen.getByRole("button", { name: "Load" }).click();
await screen.waitFor(() => {
  expect(screen.getByRole("status").text).toBe("Ready");
});
```

Publish a deterministic content-box size when a responsive branch is part of
the unit contract:

```ts
screen.getByRole("group", { name: "Panel" }).resize({
  width: 640,
  height: 480,
});
```

This drives Wabou's real `ResizeObserver` callback and Solid update; it does not
claim that Taffy would produce that geometry.

Inject typed Host capabilities without booting Rust by passing a fixture to
the component renderer. Generated DTOs remain the request and response types;
the fixture does not introduce another schema. `wabou-bindgen` emits a test
adapter beside every client, so tests implement typed DTO handlers rather than
the native JSON strings and result envelope:

```tsx
const downloads = createDownloadsTestCapability({
    list: async () => [{ id: "1", name: "demo.zip" }],
});
const host = createTestHost({ downloads });

const screen = renderComponent(() => <Downloads />, { host: host.host });
expect(host.callsTo("downloads.list")).toHaveLength(1);
```

The adapter mirrors the Rust capability boundary's `invalidRequest`,
`handlerFailure`, and `responseEncodingFailure` envelopes. TypeScript catches
DTO drift at authoring time; runtime schema validation remains Rust's job in a
native run.

Calls retain their path and exact arguments. Read-only locale/time defaults are
deterministic (`en-US`, `UTC`, 1970-01-01). Layout reads return a deterministic
1024×768 viewport with zero-sized requested nodes, allowing composed overlays
to execute in their explicit unmeasured state; pass a layout override when
positioning geometry is part of the test. Unconfigured system and font side
effects still throw instead of silently succeeding.

Components using contextual native services can receive the same typed
overrides accepted by `PlatformProvider`. This keeps clipboard, dialog,
notification, and window behavior on the production dependency-injection path:

```tsx
const screen = renderComponent(() => <DirectoryPicker {...props} />, {
  platform: {
    dialog: {
      ...testDialog,
      pickDirectory: async () => "/tmp/downloads",
    },
  },
});
```

This layer verifies component state, composition, declared roles and names,
and JavaScript event handling. It intentionally does not invent a fake layout
engine. Keep the much smaller set of geometry, native hit-testing, window, and
pixel checks in behavior or capture tests.

## Native behavior tests

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

The locator role vocabulary covers the semantic roles used by Wabou's shipped
components, including tabs, radio groups, groups, images, grids, grid cells,
menus, trees, sliders, and progress bars. These retain their native AccessKit roles
instead of degrading to a generic container. `role="presentation"` and its
ARIA alias `role="none"` remain paint and hit-test nodes but are flattened out
of the semantic tree, with their accessible descendants attached to the
nearest semantic ancestor.

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
await expect(page.getByRole("progressbar", { name: "Build" })).toHaveRange({
  value: 64,
  min: 0,
  max: 100,
});
await expect(page.getByRole("status", { name: "Save state" })).toHaveText(
  "Saved",
);
await expect(page.getByRole("button", { name: "Save" })).toBeDisabled();
await expect(page.getByRole("checkbox", { name: "Sync" })).toBeChecked();
await expect(page.getByRole("checkbox", { name: "Partial sync" })).toBeIndeterminate();
await expect(page.getByRole("button", { name: "Downloads" })).toBeCurrent("page");
await expect(editor).toBeFocused();
```

Use `toBeAbsent()` when closing an overlay or navigating away. It succeeds only
after the locator has no matching node in a completed semantic frame; it does
not treat a disabled, transparent, or merely unfocused node as absent. For an
indexed locator, absence means that occurrence no longer exists:

```ts
await page.getByRole("button", { name: "Close" }).click();
await expect(page.getByRole("dialog", { name: "Settings" })).toBeAbsent();
```

Locators are bound to the scenario runtime's `window.current` key by default.
Bind explicitly when an application opens additional native windows; recorded
and replayed actions preserve both the slot and generation:

```ts
const childPage = page.forWindow(childWindow.id);
await childPage.getByRole("button", { name: "Close" }).click();
```

Locators traverse only nodes reachable from the current accessibility root.
While an `aria-modal` dialog is active, background nodes remain available in
failure diagnostics but cannot be queried or clicked by a behavior scenario.
This matches AccessKit exposure and native modal input isolation.

Locator state assertions read the completed native semantic snapshot, not
application signals. This makes them useful for catching failures between
Solid reconciliation, Wabou's protocol, layout, and accessibility projection.
They automatically cross a native frame barrier and retry until the expected
state is visible. Use `page.waitForIdle()` directly only when a test must wait
for native layout/semantics without making an assertion. It crosses two
complete JavaScript/native frame boundaries: one can publish host inputs such
as window metrics, and the next projects the resulting Solid update into
layout and semantics. It does not wait for all animation to stop, because a
valid application may contain an infinite spinner, ripple, or activity loop.
`locator.waitFor()` uses the same native-aware retry behavior and accepts
`timeout` and `interval` options. A semantic probe reads one completed snapshot
immediately; the JavaScript runner owns retries, so there is no separate hidden
native timeout. Internal assertion probes are not added to action traces.
Locators are strict: both no match and a temporarily ambiguous match continue
polling across completed frames, but an ambiguity that survives the authored
timeout fails with its match count. Wabou never silently clicks whichever
matching node happens to appear first; give repeated controls distinct
accessible names before driving them by role.
When repetition is intentional, select a zero-based occurrence explicitly:

```ts
await expect(page.getByRole("button", { name: "Default" })).toHaveCount(2);
await page.getByRole("button", { name: "Default", index: 1 }).click();
```

`toHaveCount` polls the full unindexed role/name query and is replayable. It
accepts a non-negative safe integer, including zero. Indexed locators reject
count assertions because an occurrence represents at most one node.

The index follows depth-first logical source order rather than paint, overlay,
or z-index order. It is range-checked to an unsigned 32-bit value for
cross-platform hosts and recorded in the action trace. Presentation-only
reordering therefore does not retarget replay. An
out-of-range index waits for a dynamically inserted match until the locator
timeout; omitting the index continues to require exactly one match.
Click, keyboard, text, drag, paste, and IME actions likewise wait for an
enabled semantic target before dispatching exactly once. Wheel actions wait
for a unique present target but deliberately do not require it to be enabled:
a disabled control can still sit beneath the pointer inside a scrollable
ancestor, and disabled form state must not trap native scrolling. All actions
accept the same optional `timeout` and `interval` settings. Both deterministic
and native runs therefore use the JavaScript runner's explicit timeout instead
of backend-local implicit waits.
Locator assertions also accept `stableFor`. After the assertion first matches,
Wabou continues crossing completed native frame barriers for that duration and
resets the interval if the state stops matching. This is useful for proving
that an asynchronously refreshed keyed row retains focus without introducing
an unrecorded `setTimeout`:

```ts
await expect(page.getByRole("checkbox", { name: "Select task" })).toBeFocused({
  stableFor: 400,
  timeout: 1_000,
});
```

The resolved stability duration is stored in the action trace and enforced
during replay. It must be finite, non-negative, and no greater than `timeout`.
Drag and wheel deltas must be finite, and key actions require a non-empty key;
invalid input is rejected before it can mutate the trace or cross the native
bridge. Window ids and surface generations must also be non-negative safe
integers, and unknown key-modifier bits are rejected. The CLI applies the same
validation to imported replay artifacts.
`expect.poll` accepts synchronous or asynchronous readers and shares the same
validated timing policy. A zero timeout still performs exactly one read;
negative, infinite, and `NaN` timing values are rejected rather than producing
environment-dependent loops.

Locator snapshots also expose the logical-pixel `x`, `y`, `width`, and `height`
from the completed native semantic frame. Use `toHaveBounds` for layout
contracts that should not require GPU pixel comparison:

```ts
await expect(page.getByRole("slider", { name: "Volume" })).toHaveBounds(
  { width: 384, height: 28 },
  { tolerance: 0.5 },
);
```

The expected object may contain any non-empty subset of the four fields. Values
and tolerance must be finite; tolerance defaults to half a logical pixel. Bounds
assertions retry across native frame barriers and are preserved in replay
traces. They prove native layout geometry, not glyph rasterization, clipping
pixels, GPU output, or another platform's HiDPI behavior. Use a failure
screenshot or `wabou render` when the geometry is correct but pixels are not.

Use `toBeWithinBounds` when exact geometry is not part of the contract but a
control must remain inside a viewport, panel, or other known rectangle:

```ts
await expect(page.getByRole("textbox", { name: "Search" })).toBeWithinBounds({
  x: 0,
  y: 0,
  width: 900,
  height: 600,
});
```

Relational layout contracts should keep both elements live during polling.
`toNotOverlap` snapshots both locators after each native frame barrier and
records the second locator in the replay trace:

```ts
const identity = page.getByRole("group", { name: "Task identity" });
const status = page.getByRole("group", { name: "Task status" });
await expect(identity).toNotOverlap(status, { tolerance: 1 });
```

Touching edges do not overlap. `tolerance` permits that many logical pixels of
intersection, and both locators must belong to the same window.

Use `toHaveSameBoundsAs` for alignment and equal-size contracts without
freezing absolute coordinates:

```ts
await expect(filesTab).toHaveSameBoundsAs(overviewTab, ["y", "height"]);
await expect(secondCard).toHaveSameBoundsAs(firstCard, ["width"]);
```

The field list is non-empty and may contain `x`, `y`, `width`, or `height`.
Selected values are compared within the same logical-pixel tolerance and the
relationship remains live during polling and replay.

Containment assertions use the same logical-pixel tolerance, native-frame
polling, trace recording, and strict replay validation as `toHaveBounds`.
When the containing rectangle is the current native client area, prefer
`toBeInViewport()` so deterministic and native runs read the authoritative
logical viewport instead of duplicating the requested dimensions.

Range-bearing controls expose their numeric value independently from their
localized `aria-valuetext`. Use `toHaveRange` with any non-empty subset of
`value`, `min`, and `max` to verify the numeric contract delivered to
AccessKit:

```ts
const volume = page.getByRole("slider", { name: "Volume" });
await expect(volume).toHaveValue("100 percent");
await expect(volume).toHaveRange({ value: 100, min: 0, max: 100 });
```

Range values must be finite. Numeric comparison uses an absolute tolerance of
`1e-9` by default; pass `{ tolerance: 0 }` as the second argument when exact
identity is part of the contract. The tolerance is validated and recorded in
the trace. The assertion polls completed semantic frames, so replay verifies
both the human-readable value and the native numeric range.

Each `test` has a five-second timeout so a lost Promise cannot hang native runs
indefinitely. Override it for an intentionally long scenario with a third
argument:

```ts
test("loads a large local fixture", async ({ page }) => {
  await page.getByRole("status", { name: "Ready" }).waitFor();
}, { timeout: 15_000 });
```

Tests that exercise native file APIs can create exact UTF-8 fixtures without
depending on a developer machine's files:

```ts
test("imports a config file", async ({ page, effects, files }) => {
  const path = files.writeText("fixtures/config.toml", "enabled = true\n");
  effects.respond("dialogOpen", [path]);
  await page.getByRole("button", { name: "Import" }).click();
});
```

Fixture paths must be relative, content is limited to 16 MiB per file, and the
runner removes its isolated temporary directory when the native test host
stops. The action trace records the resulting interaction, not fixture
contents.

Timeouts report the test name and stop later test bodies from starting. The
host exits after receiving that failure report; JavaScript cannot cancel an
arbitrary Promise in place. Values must be between 1ms and 60 seconds. The
deterministic host also has a 65-second suite watchdog as a final guard for
failures that prevent JavaScript from reporting.
An empty scenario, empty test name, duplicate test name, or invalid test timeout
is a suite failure rather than a successful no-op. Registration failures stop
the suite before application state is mutated. Unexpected runner errors are
also converted into a `test runner` result instead of waiting for the host
watchdog with no diagnostic.

Name behavior files `*.behavior.ts`. Run every discovered scenario in an
application with the deterministic backend:

```bash
bun run wabou test /path/to/app
```

Wabou recursively discovers files beneath the application's `tests/`
directory and generates the bundle entry itself; no aggregate file containing
manual imports is required. To debug only one scenario, pass its path and the
application explicitly:

```bash
bun run wabou test /path/to/app/tests/window-lifecycle.behavior.ts \
  --app /path/to/app
```

The deterministic backend boots the same retained GPUI projection as a normal
application in GPUI's headless application context. It exercises QuickJS,
Solid flushes, the protocol, GPUI layout, and projected input without requiring
a compositor. It also uses an isolated temporary XDG data directory so
persisted application state cannot make scenarios order-dependent. Pass
`--native` for a real platform-window smoke test.

The deterministic backend models shell-owned window lifecycle transitions but
does not initialize native `ShellExtension` resources. Tests that must activate
an actual tray menu item or another platform callback belong in a separate
`--native` smoke suite. Keep application policy—such as the confirmation modal
shown after a quit request—in deterministic tests, and cover the extension's
callback routing with Rust tests when display-less CI cannot run the native
suite. Wabou intentionally does not fabricate an `ExtensionContext` in the
headless backend because that would make a simulated platform callback look
equivalent to the native integration it is meant to verify.

Every run writes versioned `report.json` and `trace.json` artifacts beneath
`target/wabou-test/<app>/artifacts` by default. Use `--artifacts <dir>` to
select another destination. The GPUI headless context can run without a display
server. Pixel screenshots are platform-dependent: GPUI-CE currently exposes
its headless pixel renderer on macOS Metal, while Linux wgpu runs still provide
semantic and layout artifacts but report screenshot capture as unsupported.
Pass `--failure-screenshot` to request `failure.png` where that renderer exists.
At the start of a run, Wabou removes only its known report, trace, temporary
JSON, and failure-screenshot outputs from that directory. This prevents a
build or replay-validation failure from leaving a previous green report behind
while preserving unrelated user files. JSON artifacts are published by atomic
same-directory rename so an interrupted writer cannot leave a partial report.
Failed runs also write `semantics.json`, containing the last completed semantic
tree observed for each exercised window. It includes roles, accessible names,
bounds, hierarchy, focus, and control-state flags so a missing or misrouted
locator can be diagnosed without another DevTools run. Control values are
deliberately omitted (only `hasValue` is recorded); password values are already
excluded from Wabou's semantic projection. Test action traces can still contain
authored text, paste, and IME payloads, so artifact directories should be
treated as potentially sensitive when scenarios use real credentials.
The Rust host adds the effective backend (`deterministic` or `native`), target
OS, architecture, and Wabou version to `report.json`. Failed-test console
summaries include the same compact environment line, which makes platform-only
failures identifiable without printing the full report or action payloads.
Each test result records `durationMs` and a half-open `traceStart`/`traceEnd`
range into the shared trace. This identifies the actions associated with a
failure without duplicating action payloads in each test result. The console
prints only a compact summary; the artifact files retain the complete report,
diagnostics, and action payloads.
Locator actions record their resolved timeout, polling interval, and stability
duration, and an
explicit `waitFor()` is represented as its own action rather than a one-shot
semantic probe. Replays therefore preserve authored waiting behavior even if
the framework's defaults change later. Older traces containing probe inputs
remain replayable. Newly written trace files use `{ "version": 1, "actions":
[...] }`; the CLI also accepts legacy bare action arrays, while rejecting
unknown future versions with an explicit diagnostic.
Before rebuilding the frontend or starting its Rust host, the CLI validates
every replay action and its nested input, assertion, window-state, role, and
wait fields. Corrupted or hand-edited artifacts therefore fail at the action
index that is invalid instead of surfacing later as an unrelated QuickJS or
native-window error. Unknown fields inside an action are rejected as likely
typos or schema drift; additional top-level report metadata remains allowed.
Version-one compatibility still accepts legacy bare
arrays, `probe` inputs, and locator actions written before wait metadata became
mandatory for newly recorded traces.
Semantic locator assertions are trace actions too. Absence, text, textual
value, numeric range, bounds, disabled, checked, selected, current, expanded,
pressed, and focused expectations are evaluated again during replay with their
original retry policy. A replay can therefore reproduce the failed native-state
assertion, rather than merely repeating the inputs that preceded it. Plain
JavaScript value assertions and `expect.poll` readers are not serializable and
remain report-only.
Because a replay combines actions from multiple original tests into one test,
its timeout is derived from the recorded action wait budgets instead of using
the ordinary five-second default. The derived value remains capped at 60
seconds. The complete scenario also has an explicit 60-second budget; when it
expires, the JavaScript runner records the active test name, trace range, and
stack before finishing. The host watchdog fires five seconds later so this
structured JavaScript failure wins over the native safety net. The CLI builds the Rust
host first and then supervises the resulting application process separately;
a final 70-second watchdog terminates its entire process group if synchronous
JavaScript or a broken scheduler prevents the in-process watchdog from running.

`trace.json` replays the complete action sequence:

```bash
bun run wabou test --replay target/wabou-test/app/artifacts/trace.json \
  --app /path/to/app
```

`--replay` also accepts `report.json`. Add `--replay-test` to stop after the
named test's last recorded action:

```bash
bun run wabou test \
  --replay target/wabou-test/app/artifacts/report.json \
  --replay-test "submits form" \
  --app /path/to/app
```

The replay includes the prefix from earlier tests because scenarios share one
running application. This reconstructs state established earlier in the suite
before reproducing the selected test. The CLI still validates the selected
test's half-open `traceStart`/`traceEnd` range, so a damaged report cannot hide
behind prefix replay semantics. Passing `report.json` without
`--replay-test` replays its full trace. Replay runs write to `replay-artifacts`
by default so the original failure report and trace are not overwritten; an
explicit `--artifacts <dir>` still takes precedence.

Window behavior is modeled explicitly:

```ts
test("Wayland close-to-tray", async ({ window }) => {
  await window.nativeClose(window.current, "wayland");
  await expect(window).toHaveState(window.current, {
    presence: "surface-released",
    surfaceGeneration: 1,
  });

  await window.show(window.current);
  await expect(window).toHaveState(window.current, {
    presence: "visible",
    surfaceGeneration: 2,
  });
});
```

Window-state assertions retry through the same bounded polling policy and are
recorded as replayable trace actions. This lets a close-to-tray or surface
recreation failure reproduce both the native transitions and the state that
was expected afterward. `window.state(id)` remains available for diagnostics
and custom `expect.poll` expressions.

Responsive layout can be exercised in the same scenario. Sizes are logical
pixels; deterministic tests update that window's headless viewport, while
`--native` requests a real platform surface resize. The resulting
`WindowMetrics`, layout and semantic updates remain on the ordinary frame
path, so the following locator assertion observes the completed result. Resize
actions are replayable:

```ts
test("minimum window layout", async ({ page, window }) => {
  await window.resize(window.current, 900, 600);
  await expect(
    page.getByRole("textbox", { name: "Search" }),
  ).toBeInViewport();
});
```

Use `expect.poll(() => value).toBe(expected)` for state that settles across
asynchronous host turns. DevTools remains a diagnostic interface; behavior
assertions belong in scenarios.
