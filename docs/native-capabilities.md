# Native capabilities

Wabou uses a versioned effect protocol for low-frequency operating-system
operations. Rendering keeps its compact, batched protocol; native effects do
not share the render hot path.

This is a desktop capability model, not a constrained-device RPC runtime. A
capability implementation may use the UI thread, a worker thread, native
window handles, memory-mapped files, or long-lived resources. The protocol is
the stable boundary between JavaScript and Rust, not the implementation model
inside Rust.

## Contract

An operation is identified by `(capability, method)`. Built-in capabilities
reserve low numeric IDs; application and extension crates register stable IDs
in `CapabilityRegistry`. Registration rejects collisions and records:

- the capability name and version;
- each method's UI/worker/any thread affinity;
- whether nondeterministic completion must be recorded for replay.

JavaScript submits an effect with a request ID, operation ID, scope, and typed
payload. Rust completes it with the same request and operation IDs. The public
API exposes typed wrappers such as `clipboard.readText()`, `createWindow()`,
`dialog.open()`, `notification.show()`, and `showNativeMenu()`;
`__wabou_effect_*` globals are private ABI.

## System dialogs and notifications

`@wabou/ui` exposes native dialogs as Promise-based effects. Cancellation is
a normal result rather than an exception: file and directory methods resolve
to `null`. Platform or permission failures reject the Promise.

```ts
import { dialog, notification } from "@wabou/ui";

const files = await dialog.open({
  multiple: true,
  filters: [{ name: "Images", extensions: ["png", "jpg"] }],
});

const destination = await dialog.save({ defaultName: "export.json" });
const directory = await dialog.pickDirectory({ title: "Choose a workspace" });
const answer = await dialog.message({
  title: "Discard changes?",
  message: "This cannot be undone.",
  level: "warning",
  buttons: "yesNoCancel",
});

await notification.show({
  title: "Export complete",
  body: destination ?? "The export was cancelled",
});
```

Dialogs are parented to the requesting Wabou window. The current native
backend uses each platform's file/message dialog through `rfd`; Linux uses the
XDG desktop portal by default. Notifications use the operating-system
notification service through `notify-rust`. Native backend types and handles
do not cross the effect protocol.

`useDialog()` and `useNotification()` read `PlatformProvider` overrides, so
component tests can inject deterministic services without opening native UI.

External packages can build equally typed wrappers with `EffectOp` and
`dispatchEffect`. The Rust side accepts extension payload bytes, so adding a
capability does not require editing a central enum. Public packages should
generate their DTO declarations from their Rust wire types and keep a small
hand-written ergonomic API above them.

## Three kinds of native work

- Pure synchronous queries may eventually use a dedicated sync path, but must
  be fast and reentrant. Effects are async by default.
- One-shot effects complete a Promise: clipboard, dialogs, native menus, and
  window commands.
- Long-lived resources return a stable handle and publish events: windows,
  file watchers, tray items, processes, and streams. Bulk bytes should move by
  shared buffers or resource handles rather than JSON.

The first implementation covers one-shot effects and window resource handles.
The operation and registry design leaves sync queries, streams, and shared
buffers additive instead of requiring another ABI rewrite.

## Record and replay

Replay applies to nondeterministic native boundaries, not to the whole desktop
application or render loop. `RecordingEffectExecutor` records request and
completion pairs. `ReplayEffectExecutor` verifies request order and identity,
then returns recorded completions without touching the OS. Pure rendering and
deterministic JavaScript continue to run normally.

Secrets require an explicit recording policy. A password manager should mark
credential and clipboard payloads as redacted or non-recordable rather than
silently writing them into a trace.

`HostBuilder::record_effects(path)` records only payload-safe window state
commands. `record_all_effects(path)` is an explicit opt-in that may persist
clipboard text, menu labels, window titles, or third-party payloads.
`replay_effects(path)` intercepts operations present in the tape and lets
unrecorded operations execute live. The tape is ABI-versioned, shared across
all windows, preserves asynchronous completion order, and remaps recorded
request IDs to the current run.

## Current limitations

- Notification actions and activation routing are not part of the initial
  common API. `notification.show()` currently covers title, body, icon and
  silent delivery only. macOS does not permit applications to replace the
  notification icon; Windows notification identity follows the configured
  window/application name until packaging provides a dedicated AppUserModelID.
- Native dialogs are modal. Their JavaScript API remains asynchronous, but the
  shell event thread waits inside the platform dialog's own modal event loop.
- Context-menu selection is implemented by `wabou-tray`; dismissal detection
  is not portable through `muda` yet, so `showNativeMenu()` currently resolves
  only after an item is selected.
- Extension payloads currently cross QuickJS as JSON bytes. Large-buffer and
  stream transports remain follow-up work.
