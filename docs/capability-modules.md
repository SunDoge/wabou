# Replaceable native capability modules (experimental)

The `wabou-runtime/capability-modules` feature provides a stable dispatcher for
trusted application libraries. The runtime, QuickJS capability objects and host
state remain resident. A `NativeCapability::module_method` wrapper selects the
current native implementation on each invocation; it is mounted only once.

The CLI's `--rust-hot-reload` option now starts a Cargo-based stable host and opts
applications into a module development service. Gallery uses this path for palette
behavior. Other applications need to register their own module services. The old
Subsecond adapter has been removed. `wabou-capability-sdk` supplies ordinary Rust
adapters for the current native ABI; that ABI implementation remains experimental.

## Ownership and replacement

`CapabilityModule::new(contract_version, host_operations)` owns a reference to
stable host operations. Put durable application state, connections, resource
registries and task scheduling behind that host closure. A module accesses them
through operation names and encoded resource identifiers, not borrowed Rust
objects. The host closure and its captured state survive `reload`.

`reload` is unsafe: libraries must satisfy the documented native ABI contract.
They are trusted code, not a sandbox. A candidate is copied into a private temporary
directory, loaded and checked before replacing the current generation. Failure
leaves the old implementation installed. The source library remains owned by the
build workflow; the dispatcher deletes its private copies as generations retire.

An invocation pins its starting generation. Replacing the dispatcher does not
redirect pending work into new code. Completion, failure and cancellation invoke
the starting generation's destructor before its library can unload. Dropping an
invocation cancels it. A completed invocation retained by its caller still pins
the generation until that invocation object is dropped.

## ABI and asynchronous work

`wabou-capability-abi` uses stabby 72.1.16 to define and report the stable
layout of buffers, callbacks and the module function table. ABI version and application contract version are separate. Payloads
are currently JSON at the native module boundary; JS still receives structured
values through `module_method`. Changing a DTO contract requires a new contract
version and is rejected by a dispatcher expecting the old version.

Modules export `wabou_capability_v2` returning `ModuleApi`. Its `start`, `poll` and
`drop_task` functions create, advance and destroy an opaque module-owned invocation.
All byte slices, output callbacks and host contexts are borrowed for that single
ABI call. They must not be retained. Rust allocation ownership, futures, trait
objects and QuickJS handles do not cross the boundary. Panics must be contained
inside the module or abort; unwinding across the boundary is forbidden.

The host initially polls pending invocations every 2 ms. Module tasks remain on
their starting thread. Long-running work should be scheduled in the stable host
and queried through resource/task identifiers. Native threads or callbacks owned
by a module must stop before `drop_task` returns. In particular, arbitrary Rust
futures whose wakers escape into another executor are not supported: those wakers
may contain pointers into a library after cancellation. This requires explicit
lifetime support before an ordinary async-function SDK can be safe.

The current output limit is 1 MiB per start/poll output. This is not a resource
security boundary; a trusted native library can access the host process directly.

## Evidence

```sh
cargo test -p wabou-runtime --features capability-modules capability_module::tests --lib
```

These tests compile and load actual shared libraries. They check retained host
state, pending old-generation work, new-generation dispatch, rejected replacement,
exactly-once cancellation and private-library cleanup. The QuickJS integration
retains the original JS function reference across replacement. Another test builds
the actual Gallery palette implementation, edits a temporary copy and verifies
its new response through the unchanged JSON capability. A host-owned background
job/resource test verifies that work starts once and old/new module invocations
resolve through the same resource identifier after replacement.

The native-host registration path also has a deterministic behavior scenario:

```sh
cargo run -p wabou-cli -- test apps/gallery/tests/capability-module.behavior.ts --app apps/gallery --env WABOU_RUST_HOT_RELOAD=1 --artifacts /tmp/wabou-module-behavior
```

This exercises Gallery's real HostBuilder service startup, Cargo module build,
QuickJS capability and generated JS client. It does not synthesize desktop input
or prove platform-specific loader behavior, visual correctness or arbitrary
escaping future/waker lifetimes. Most existing applications still use ordinary
linked handlers; migrating them is optional and requires defining module boundaries.

## Writing a module in Rust

The SDK supports a small `Module` trait, a per-invocation `Task` trait and a
borrowed `Host` context. `Ready::json` wraps bounded synchronous results. A task
can return `Poll::Pending` while waiting for runtime-owned work; it receives a
fresh host context on each poll. `Host::call_json` accesses stable operations
without passing Rust references across libraries. `export_module!(MyModule)`
exports the function table. The SDK contains unwinding panics during start, poll
and destruction; a `panic=abort` build still aborts the process.

The counter example separates durable counter storage from reloadable behavior:

```sh
cargo build -p wabou-capability-sdk --example counter
cargo run -p wabou-runtime --features capability-modules --example capability_module -- target/debug/examples/libcounter.so
```

Use `libcounter.dylib` on macOS or `counter.dll` on Windows. This example explicitly
loads the trusted SDK library twice, increments the same host-owned counter across
replacement and asks module code to format its label. It does not open a GUI or
watch source edits. The SDK adds no support for escaping futures or wakers; the
asynchronous lifetime restrictions above still apply.

## Host-owned development service

Enable `wabou/capability-modules` and register a `CapabilityDevelopment` with
`HostBuilder::service`. Mount its `CapabilityModule` through `module_method` once.
The service builds and loads the first implementation before window runtimes are
created, watches source content every 200 ms, and uses PATH `cargo build --lib`
for later edits. Failed builds/reloads retain the old implementation; a corrected
edit retries. Source edits during a build are detected in the next iteration.
`status()` reports completed attempts and the last failure.

```rust,ignore
let module = CapabilityModule::new(CONTRACT.version(), host_operations);
let mut project = ModuleProject::new("app-functions/Cargo.toml");
project.watch.push("app-model/src".into()); // relevant local dependency
let development = unsafe { CapabilityDevelopment::new(module.clone(), project) };
HostBuilder::new()
    .service(development)
    .capability(CONTRACT, move |capability| {
        capability.module_method(METHOD, module.clone())
    })
    .run()?;
```

Use a trusted module project: loading arbitrary native code is unsafe. Initial
build failure currently prevents service startup; recovery while the app is
running applies to subsequent edits. The module must have a cdylib library target.
The library path comes from Cargo's artifact output, respecting target directories
and platform suffixes. Cargo retains its normal incremental build cache; the
runtime only retains private copies of live library generations.
Completing or cancelling an invocation clears both its generation pin and its
host-operation reference immediately. Retaining the finished invocation object
does not retain an old library, its private copy or an otherwise retired host
namespace. Cleanup executes before those references are released.

Independent builders can set `ModuleProject::target_directory` (or Project JSON
`targetDirectory`) to isolate Cargo outputs. The default preserves Cargo/mbx's
configured shared cache. Concurrent integration tests give each temporary project
its own target directory, preventing same-named fixture libraries from overwriting
each other between compilation and loading. These directories are removed with
the temporary projects. An explicitly configured output directory is excluded
from watching even if its basename is not `target`.

Default watching covers the module directory and excludes target, .git,
node_modules and dist. Symlinks are skipped. Explicitly add local dependency roots
and relevant workspace manifests/configuration outside that directory. Watching
uses native filesystem notifications with a 75ms debounce and content hashing to
avoid rebuilding after unchanged or excluded-file events. A 30-second safety scan
catches missed events; unavailable or failed native watchers fall back to 200ms
polling. Shutdown wakes the watcher, terminates its
current Cargo command and joins the worker thread. It does not yet manage Cargo
build-script descendants as an OS process group.

## Typed handler convenience

The resident runtime's task service notifies waiting capability invocations when
work completes. Invocations reporting a pending `task.poll` wait on that notification
instead of polling every 2ms. Completion registration precedes polling to avoid a
lost-wakeup race. Custom async implementations that do not use the resident task
service retain the 2ms fallback. Neither futures nor notification objects cross
the module ABI.

For bounded handlers the SDK can own request decoding, response encoding and the
invocation adapter. Keep application functions in ordinary Rust:

```rust,ignore
pub fn describe_palette(request: DescribePaletteRequest)
    -> Result<DescribePaletteResponse, String>
{
    // Normal application code, independently testable.
}
export_handlers! {
    version: CAPABILITY.version(),
    DESCRIBE_PALETTE => |_, request| describe_palette(request),
}
```

The first closure parameter is an explicit borrowed Host, usable by handlers that
need runtime-owned state/resources. Only registration uses a small macro; function
bodies stay outside it for normal formatting and LSP behavior. Custom Task remains
available for multi-step work. This adapter does not turn arbitrary escaping async
futures into safely unloadable tasks. See [runtime distribution](runtime-distribution.md)
for the separate precompiled runtime/launcher prototype using stabby.

## Resident resource providers

The host can keep native objects in
`capability_resources::CapabilityResources<Family, Value>`. This reuses Wabou's
generational `ResourceRegistry`: insert returns a typed `ResourceKey<Family>`,
remove invalidates that generation, and `with` resolves only live handles.
Construct objects and their destructors in resident host code. Do not store
replaceable-module objects, callbacks or vtables in this registry.

Modules use the SDK's reexported `ResourceKey<Family>` and
`host.resource_call("connection.read", key, &request)`. Providers receive
`{ "resource": { "lo": ..., "hi": ... }, "request": ... }` and validate the
handle in their own registry. Family branding is a compile-time convenience,
not wire authentication or a sandbox. Each provider/namespace must resolve only
its own resources. Registry callbacks hold a lock: keep them bounded and avoid
reentry; clone an Arc for work that runs asynchronously.

The actual SDK library replacement test preserves a resource handle inside typed
state, reads the same host-owned Arc before/after replacement, verifies old code
is released, and verifies calls fail after host removal. The file provider described below is integrated into the shared runtime;
other resource providers remain to be added.

## Resident asynchronous jobs

`capability_tasks::CapabilityTasks` schedules ordinary host-owned futures on a
resident Tokio handle and returns `TaskKey`. The future, its captures and all
destructors must be compiled into resident code. Module implementations never
give this executor futures or wakers. Providers can start network/file work and
return the resulting key using their normal stable operations.

The SDK offers `host.task_poll::<Response>(key)` and `host.task_release(key)`.
Results are copied and remain readable until release, allowing old/new invocations
to observe one background job across replacement. Releasing a job invalidates
the handle and aborts pending work; dropping the final scheduler aborts all jobs.
Abort is asynchronous/cooperative, so the executor must remain alive to finish
dropping futures. It cannot interrupt an already-running blocking operation.
Custom invocation tasks may override `Task::dispose(&Host)` to release their
invocation-owned jobs/handles on completion, failure or cancellation. Cleanup
runs before the originating library is released, using a fresh borrowed context.
Leave durable resources alive intentionally. Cleanup and Rust destruction use
separate panic guards so cleanup failure does not skip destruction.
Completed handles must be released to reclaim result storage. Panics in a host
job become failed snapshots when unwinding is enabled; panic=abort terminates
the process.

The shared runtime supplies one resident executor and a separate task namespace
per capability. Its built-in `host.sleep(duration)` schedules a host timer,
polled as `task_poll::<()>`. Application-specific async providers still need
registration. The actual dylib replacement test uses this scheduler: old/new
module calls wait on the same host future and receive their own version's result,
with the old generation kept alive until its invocation finishes.


## Resident files

The shared runtime supplies `CapabilityFiles` in each capability namespace.
`host.file_open(path)` returns a task polled as `FileKey`; `host.file_read(key,
length)` returns a task polled as `Vec<u8>`. Release task handles after reading
their results, and call `host.file_close(key)` when the file is no longer needed.
Keep the file key in runtime state when it should survive module replacement.
Files are read-only in this initial provider.

Relative paths use the launch configuration's `resourceDirectory`, defaulting
to the configuration directory. This is resolution, not a filesystem sandbox.
Open/read work runs in resident Tokio code. Reads serialize per file and advance
its resident cursor, can be shorter than requested, and are limited to 64 KiB
per call to bound the JSON response. Closing prevents new reads, while existing
reads retain an Arc and may finish. Cancelling I/O cannot promise cursor rollback
or interrupt underlying blocking OS work. A successfully opened resource belongs
to the namespace until explicitly closed or its owner shuts down; releasing the
open task does not close the file, including if its completed result was never read.

The actual SDK replacement test stores a file handle in typed state, reads `A`,
replaces the implementation, and reads `B` from the same file/cursor. A focused
provider test verifies an already-started read completes after close, future
reads reject the stale handle, and oversized chunks are rejected.


## Resident TCP connections

The shared runtime supplies `CapabilityTcp` per capability. The SDK exposes
`tcp_connect` (task result `ConnectionKey`), `tcp_read` (task result `Vec<u8>`),
`tcp_write` (task result `usize`) and `tcp_close`. Store the connection key in
runtime state to reuse it after replacement. Release I/O task handles after
reading their results, and close the connection when its owner no longer needs it.
Successful connection resources remain resident until close/namespace shutdown;
releasing a completed connect task does not close its connection.

Read/write halves use independent locks, so a pending read does not block writing.
Operations use copied chunks of at most 64 KiB and can return partial reads/writes.
After a successful write, retry only the unreported remaining bytes. Closing
invalidates the handle and signals pending I/O to fail with `connection closed`;
cleanup is cooperative on the resident executor. Dropping the final connection
registry also signals outstanding operations. The provider is plain TCP, with no
framing or TLS, and is for trusted native modules.

A loopback provider test starts a read before writing, verifies the echo, closes
with another read pending, checks its failure and rejects later use of the handle.
The real dylib replacement/background-task test also uses a loopback connection:
old/new calls wait on one host job, retirement leaves its native stream alive, and
the server accepts only one connection. It does not reconnect during replacement.


## Architecture verification

| Requirement | Implementation and direct evidence |
| --- | --- |
| Resident QuickJS/window/layout/rendering runtime | Separate runtime cdylib and dependency-light launcher; real deterministic shared-runtime launch scenario |
| UI implementation replacement | Vite HMR test retains the same state object/text and native function while executing the new view |
| Stable capability entry | QuickJS and Gallery SDK dylib tests retain a saved JS function across actual library replacement |
| Host-owned state and resource identity | Typed-state test preserves count/text and file cursor across SDK replacement; loopback task test preserves one native connection |
| Resident task scheduling | Tokio job registry and typed keys; loopback native scenario covers timer, file and TCP tasks through QuickJS and SDK |
| Safe generation retirement under the trusted module contract | Pending calls pin old code; completion/cancellation clean up before releasing the library; retained finished handles do not retain retired namespaces/private copies |
| Failed compilation preserves current application code | Cargo watcher test introduces broken Rust, verifies old dispatch, then verifies recovery after correction |
| Module rather than mandatory actor as replacement unit | SDK Module/Task function table and handler registration; no actor requirement in application-function modules |
| Application behavior moves out of fixed host | Gallery palette behavior lives in gallery-functions; real generated-client test observes its changed implementation without remounting capability |

Latest focused validation on Linux x86_64: 122 runtime tests, 3 SDK tests, 82 CLI
tests, and two explicit prebuilt launch tests passed. The native service scenario
checks the named deterministic report, not merely process exit. This validates
the runtime/module architecture, not arbitrary native plugins, every existing
application's adoption, GPU pixel correctness or a macOS/Windows distribution.

Development status retains rustc's rendered error diagnostics, including source
locations, when a capability compilation fails. The watcher keeps the old
implementation installed and retries after the next source edit. Cargo stderr
remains the fallback for failures outside rustc (for example dependency resolution).

The loader checks stabby's complete entry signature before calling the candidate.
ABI v2 removes the manual table-size field; the application contract version
is still checked separately. ABI v1 libraries must be rebuilt. The SDK generates
the pinned stabby checked-export protocol hygienically, so applications do not
need a direct stabby dependency to expand `export_module!`. Static linking uses
the same table through ordinary Rust calls and needs no dynamic signature lookup.
