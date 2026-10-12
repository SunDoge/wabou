# Rust capability + widget hot reload

From the repository root:

```sh
bun run wabou dev examples/widget-reload
```

The CLI contains the resident runtime. On first use it compiles the small
`native` cdylib; subsequent saves rebuild just this implementation and the SDK.
Click the native counter or the Rust capability button, change `STEP` or a
color in `native/src/lib.rs`, and save. The count, node identity and native focus
survive. A broken Rust edit
leaves the previous library running. Fix the edit to resume replacement.

`wabou.config.ts` uses `rustPlugin` to register one project with capability
contracts and widget tags, initial states and fixed schema versions.
Compose it using `NativeWidget` in TSX. Each native node owns a separate state
slot, even when multiple nodes use the same library. UI edits still use Vite HMR;
removing/recreating a node destroys its native state as usual.

Implement `Module` for capabilities and `WidgetImplementation` for widgets.
Export them together once:

```rust
export_plugin! {
    capabilities { "counter" => Counter }
    widgets { "reload-counter" => CounterWidget }
}
```

The example exports two capabilities and two widgets; the configuration selects
one of each. Selected consumers share one library generation and one watcher,
while capability host state and each widget instance remain independent.
Generated capability contracts can be passed directly to `rustPlugin` to avoid
repeating the method list. Legacy `export_module!`, `rustCapability` and
`rustWidget` remain available for single-export libraries. Widget callbacks are synchronous and exchange JSON data over the
existing stabby capability boundary. `paint` returns rectangle/text commands;
the resident renderer owns font shaping, clipping, scale and Vello resources.
Drawing is cached until state/config/size/focus or implementation changes.
State is **not** serialized or recreated on reload; callback inputs currently
copy/encode it. This is intended for small custom controls, not large editors or
per-frame large data sets. Events include pointer transitions/movement, wheel, keyboard, committed text,
paste, focus and IME lifecycle. Text widgets opt in with `accepts_text_input`.
Arbitrary native widget APIs, GPU resources and async widget callbacks are not
supported by this initial protocol.

The loaded candidate is published transactionally. Before publication the host
reads its registry and checks every selected export and contract/schema version.
A missing or incompatible selected export rejects the whole replacement,
including otherwise compatible capabilities. Widgets pin their current
implementation until the next host poll boundary; old libraries stay loaded
until their last widget/invocation releases them. Failed callbacks retain the
last valid state/drawing. Changing schema version is rejected rather than
silently resetting state; migrations are not implemented yet.

For production, `wabou bundle examples/widget-reload --out <new-directory>`
builds and packages the combined plugin as one shared library. Add `--static` to link the runtime and plugin registry into the application
executable. The native crate must provide `rlib` alongside `cdylib`; this example
already does. Static packaging still ships frontend resources beside the executable.

Build artifacts are shared in `target/widget-reload-demo`. The existing loader
keeps one private copy per live implementation and removes retired copies;
there is no permanent per-save library archive.

The automated test builds real shared libraries and verifies event/paint code
replacement, state slot address/count/config/focus preservation, pending-swap
frame consistency, wake/layout invalidation and failure fallback:

```sh
cargo test -p wabou-shell-vello --features widget-hot-reload hot_reload --lib
```

Check and exercise the complete plugin application without opening a desktop window:

```sh
bun run wabou check examples/widget-reload
bun run wabou test examples/widget-reload
```

`build` and `run` also use the CLI's resident runtime for this app. They compile
selected plugin implementations, then load the frontend bundle; no application
runtime crate is built. The example accesses its typed capability through
`useHost` and validates its version using `bindCapability`.
