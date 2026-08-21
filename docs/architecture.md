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
  Rust runtime (layout, input, resources, semantics)
                       |
       renderer and platform-specific shell
```

## Ownership

Reusable UI composition follows the stricter
[component composition contracts](component-contracts.md): surface, focus,
clip, scroll, overlay, semantics, and native content each have an explicit
owner.

JavaScript owns application state, component composition, interaction policy,
semantic intent, and routing. A primitive must author capabilities such as
focus participation explicitly. Rust does not infer application behavior from
HTML conventions, tag names, `href`, or CSS classes.

Rust owns validation and execution: the retained node tree, layout, clipping,
hit testing, focus routing, accessibility projection, resources, painting,
window lifecycle, and operating-system integration. Native widgets may provide
intrinsic size, painting, input, and semantic data through their typed widget
contract; they do not create hidden JavaScript state.

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
that prevents cycles. For example, `wabou-shell` owns the widget contract,
`wabou-widgets` implements it without depending on the runtime, and
`wabou-host-api` is shared by runtime and binding generation. A new crate must
demonstrate one of those compile or dependency boundaries; ordinary subsystem
ownership belongs in a module. Applications still see the `wabou` facade.

## Cross-language contract

Wabou has four communication mechanisms. Their normative selection and
resource-lifetime rules live in [the runtime boundary contract](runtime-contract.md).

| Mechanism | Purpose |
| --- | --- |
| frame protocol | high-frequency, batched mutation and host-event data |
| native intrinsics | private synchronous runtime and engine primitives |
| native capability | direct typed application request/response APIs |
| JSON capability | low-frequency application request/response APIs |

Long-running application producers publish through the host event frame; they
do not invent another callback ABI. Native effects are not an application
plugin mechanism: raw numeric effect operations remain internal to
`@wabou/core`, while applications use typed capabilities and host messages.
New cross-language features must have one authoritative declaration
and generated Rust/TypeScript views; handwritten parallel enums or registration
lists are drift bugs.

The runtime and default `wabou` facade consume lightweight `JsonMethod` and
`HostMethod` contracts. Specta and the TypeScript exporter remain behind
`wabou-bindgen`'s `generate` and the facade's `bindings` features, so executing
an application does not inherently depend on code-generation machinery.
Applications may mount direct structured-value methods through
`HostBuilder::native_capability`; JSON remains the default for low-frequency
control operations, while native capabilities serve measured hot calls and
stable typed object operations.

The protocol transports explicit facts. For example, JS sends focusability and
focus order as an interaction policy. Rust validates and applies that policy;
it does not derive focusability from a button-like role.

### Future host actors

TODO: evaluate Kameo as the runtime behind long-lived application services.
Wabou would keep ownership of the public contract layer: explicitly exported
ask, tell, and event messages would carry stable names and versions, feed
Specta/TypeScript generation, and map onto JSON capabilities and host messages.
Internal actor messages would remain ordinary Rust types and would not become
part of the JavaScript API. The experiment must demonstrate that it removes a
real hand-written command loop without duplicating `HostService` lifecycle,
cancellation, or shutdown semantics before it becomes framework API.

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
