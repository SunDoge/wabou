# Architecture boundaries

Wabou is internally modular but presents one application model. Ordinary
applications use `@wabou/ui`, `@wabou/vite`, `@wabou/test`, and the Rust
`wabou` facade. Components, primitives, animation, and routing are source
directories inside the UI package rather than additional workspace packages;
repository topology is not an installation contract.

```text
Application (Solid state and explicit UI intent)
                       |
               @wabou/ui facade
                       |
       generated, versioned Wabou operations
                       |
     Rust runtime (protocol and native services)
                       |
          retained GPUI-CE element projection
```

Wabou applies each completed Solid flush to a retained GPUI-CE tree. GPUI owns
layout, text, painting, native input, and platform windows; Wabou owns the
versioned operation protocol, explicit application semantics, resource handles,
and the projection into GPUI elements. There is no renderer feature switch and
no lowest-common-denominator backend interface. The retired Winit/Vello system
is isolated in unpublished `wabou-legacy-*` crates as a migration oracle.

## Ownership

Reusable UI composition follows the stricter
[component composition contracts](component-contracts.md): surface, focus,
clip, scroll, overlay, semantics, and native content each have an explicit
owner.

JavaScript owns application state, component composition, interaction policy,
semantic intent, and routing. A primitive must author capabilities such as
focus participation explicitly. Rust does not infer application behavior from
HTML conventions, tag names, `href`, or CSS classes.

Rust owns validation and execution: the retained node projection, resources,
window lifecycle, and operating-system integration. GPUI executes layout,
clipping, hit testing, focus routing, text and painting. Native widgets are
application-defined GPUI elements with optional retained GPUI entities; they do
not create hidden JavaScript state.

Some inference remains local to a subsystem rather than crossing this
boundary. Examples include accessibility deriving a label from explicit text
descendants and layout resolving intrinsic sizes. These operations interpret
an already-declared tree; they do not invent interaction behavior.

## Public surfaces

- `@wabou/ui` is the default JavaScript import and JSX runtime. It exposes
  styled components, common scene primitives, routing, animation, and native
  host services. `@wabou/ui/primitives` is the explicit lower-level escape
  hatch.
- `@wabou/vite` owns build integration and static style compilation.
- `@wabou/test` owns native behavior testing. `wabou test <app>` discovers
  `tests/**/*.behavior.ts`; applications do not maintain an import registry.
- `wabou` is the Rust application facade. Runtime, shell, accessibility, style,
  and widget crates can remain separate for compile-time and ownership reasons
  without becoming normal application dependencies. Optional extension crates,
  such as the terminal widget or bindgen tooling, are explicit additions rather
  than alternate runtime entry points. The facade exports its application API
  explicitly; renderer internals such as `Applier`, `JsRuntime`, protocol
  decoders, and HMR machinery remain implementation details. Native widget
  authors use the deliberate `wabou::widget_api` extension surface.

## Physical boundaries

A source ownership boundary does not automatically become a package or crate.
JavaScript has five installable units because each has a distinct consumer or
lifecycle: application UI (`@wabou/ui`), embedded runtime (`@wabou/core`),
build integration (`@wabou/vite`), behavior tests (`@wabou/test`), and the
optional terminal widget (`@wabou/terminal`). Component, primitive, animation,
routing, protocol, renderer, and style code remain directories inside their
owning package. The package check rejects retired implementation package names
so this graph cannot grow back accidentally.

Rust crates may remain narrower when they isolate a large dependency family,
an optional extension, a platform/tooling target, or a dependency direction
that prevents cycles. For example, `wabou-shell` owns the GPUI projection and
native-widget mounting contract, while `wabou-host-api` is shared by runtime and
binding generation. `wabou-legacy-*` crates are excluded from this production
graph. A new crate must demonstrate one of those compile or dependency
boundaries; ordinary subsystem ownership belongs in a module. Applications
still see the `wabou` facade.

Repository verification follows the same boundary. Ordinary `verify:rust` and
CI commands operate on Cargo's formal `default-members`, so they do not compile
Winit, Vello, or AnyRender through the migration oracle. Use
`bun run verify:legacy` explicitly when changing or comparing the retired
implementation. The architecture check rejects any dependency from a
non-legacy workspace member back into that graph.

## Cross-language contract

Wabou has four communication mechanisms. Their normative selection and
resource-lifetime rules live in [the runtime boundary contract](runtime-contract.md).

| Mechanism | Purpose |
| --- | --- |
| frame protocol | high-frequency, batched mutation and host-event data |
| native intrinsics | private synchronous runtime and engine primitives |
| capability | typed application request/response APIs; JSON is an optional method codec |

Long-running application producers publish through the host event frame; they
do not invent another callback ABI. Native effects are not an application
plugin mechanism: raw numeric effect operations remain internal to
`@wabou/core`, while applications use typed capabilities and host messages.
New cross-language features must have one authoritative declaration
and generated Rust/TypeScript views; handwritten parallel enums or registration
lists are drift bugs.

The runtime and default `wabou` facade consume lightweight `JsonMethod` and
`HostMethod` contracts. Both are methods in one versioned capability namespace:
`HostMethod` exchanges structured QuickJS values directly, while `JsonMethod`
opts into JSON text for dynamic or externally sourced payloads. Specta and the
TypeScript exporter remain behind `wabou-bindgen`'s `generate` and the facade's
`bindings` features, so executing an application does not inherently depend on
code-generation machinery. Applications mount the namespace through
`HostBuilder::capability`; JSON is a codec choice, not a second capability
system.

The protocol transports explicit facts. For example, JS sends focusability and
focus order as an interaction policy. Rust validates and applies that policy;
it does not derive focusability from a button-like role.

### Application workers and caches

`SerialWorker<Request, Response>` covers the common case where one native
thread must exclusively own an inference engine, parser, or other thread-affine
resource. It initializes state on the named worker thread, processes a bounded
FIFO queue, returns typed results through an async request, and joins during
shutdown. Queue saturation is an explicit error; it never blocks the UI thread.
This is deliberately smaller than an actor framework: public JS contracts still
belong to capabilities and host messages, not worker-internal command enums.

`PersistentJsonCache` stores low-frequency, reproducible application results
separately from renderer assets. It provides boundary-preserving content keys,
atomic immutable writes, malformed-entry recovery, path-safe namespaces, and a
bounded disk directory. API keys and other credentials must not participate in
cache keys or values.

The Manga OCR app is the reference integration: separate serial workers own its
OCR and LLM state, while OCR, translation, and vision-adjusted bbox results use
one content-addressed persistent cache. This removed the app's hand-written
channel, oneshot, thread, and temporary-file lifecycle code without introducing
an actor runtime.

TODO: evaluate a larger actor framework only when an application needs dynamic
object discovery, supervision, or many independently addressable workers.

## Tooling contract

The CLI is the orchestration boundary. `wabou dev`, `test`, `build`, `package`,
and `doctor` must hide package builds, code generation, frontend bundling, host
compilation, process cleanup, and diagnostics whenever those steps are
mechanical. It invokes project-local tools directly rather than treating
package-script names as a framework protocol. A command may ask for an
application decision, but should not make the user reproduce Wabou's internal
package graph.

## Review questions

When adding a feature:

1. Can an application reach it through the facade without learning an internal
   package or crate?
2. Is intent authored once in JS and executed predictably in Rust?
3. Does cross-language data have one source of truth?
4. Can `wabou test <app>` discover and verify it without another registry?
5. Is visual or platform behavior verified at the layer where it can fail?
6. Does cross-language work follow the frame/intrinsic/capability selection
   rule, with Rust-owned resources using typed generational handles?
