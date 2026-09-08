# Architecture boundaries

Wabou presents one application model: Solid 2 runs inside QuickJS and emits a
versioned mutation protocol into a retained native runtime. The current backend
uses Winit for windows and input, Taffy for layout, Parley for text, and
Vello Hybrid for painting.

```text
Application state and TSX (Solid 2)
                 |
          completed Solid flush
                 |
     batched, versioned operations
                 |
        retained Rust projection
       /          |          \
    Taffy       Parley    native widgets
       \          |          /
          Vello Hybrid scene
                 |
               Winit
```

The repository previously contained GPUI and AnyRender experiments. They are
not application backends or public API. Backend-specific names remain only
where a crate owns a real implementation boundary, such as
`wabou-shell-vello` and `wabou-terminal-vello`.

## Sources of truth

Wabou keeps one authoritative owner for each kind of state:

- Solid signals and owners own application state and authored component
  structure.
- `packages/core/src/protocol/index.ts` owns the binary wire format. Generated
  Rust constants and golden frames verify it; they are not separate schemas.
- The committed Rust projection tree owns the latest native node structure. A
  partially decoded frame is never observable.
- Taffy owns completed layout. The shell owns clipping, hit testing, focus,
  window integration, and accessibility publication.
- A focused native editor owns transient selection and IME composition. Solid
  receives committed values and remains the owner of durable application data.
- Native widget instances own only their local measurement, paint, and
  interaction state.

Component tests can prove JavaScript composition and declared behavior. Layout,
native input, and pixel claims require the corresponding native test layer; a
JavaScript-only mock is not evidence for them.

## Protocol and invalidation

The opcode stream describes retained UI intent rather than renderer calls. Node
creation, attachment, text, style, listeners, resources, and imperative
commands remain stable cross-language facts. Rust classifies changes as
structure, layout, paint, interaction, semantics, or widget work and invalidates
only the necessary native phase.

Solid batches mutations at its flush boundary. Host events are batched into
versioned frames in the opposite direction. Neither side calls through while
the retained tree or a widget is mutably borrowed.

Add an opcode only when the retained model cannot represent a stable fact.
Platform handles, Taffy nodes, Vello scene objects, and widget implementation
details never cross the wire.

## Ownership

JavaScript owns application state, component composition, routing, interaction
policy, and semantic intent. A primitive declares capabilities such as focus
participation explicitly. Rust does not infer web behavior from tag names,
`href`, or CSS conventions.

Rust validates and executes that intent: retained nodes, resources, layout,
text, paint, native input, windows, and operating-system services. Inference is
limited to local native work such as intrinsic measurement or deriving an
accessible label from explicit descendants.

Native widgets follow the same split. Solid authors a complete configuration
snapshot; the widget measures, paints, and handles transient input; typed
events return facts to Solid. See [native widgets](native-widgets.md).

## Public surfaces

- `@wabou/ui` is the application-facing JSX and component API.
- `@wabou/core` contains the embedded renderer and protocol runtime.
- `@wabou/vite` owns build integration and static style compilation.
- `@wabou/test` owns component, layout, native behavior, and capture tooling.
- `@wabou/terminal` is the optional terminal component package.
- `wabou` is the Rust application facade. Applications normally use
  `wabou::HostBuilder` and do not depend on backend internals.

The five JavaScript packages have different consumers and build lifecycles.
Components, primitives, animation, routing, protocol, and style subsystems stay
as source directories inside their owning package rather than becoming more
installable packages.

Rust crates may remain narrow when they isolate a large dependency family, a
platform boundary, an optional extension, or a dependency direction that
prevents cycles. A new crate must demonstrate one of those boundaries; ordinary
subsystem ownership belongs in a module.

## Cross-language contract

The normative selection and lifetime rules live in
[the runtime boundary contract](runtime-contract.md).

| Mechanism | Purpose |
| --- | --- |
| frame protocol | high-frequency batched mutations and host events |
| native intrinsics | private synchronous runtime/engine primitives |
| capability methods | typed application request/response APIs |

`HostMethod` exchanges structured QuickJS values directly. `JsonMethod` is the
same capability mechanism with JSON as an explicit codec for dynamic or
externally sourced payloads; it is not a second plugin system. Long-running
native producers publish through host event frames.

New cross-language features need one authoritative declaration and generated
Rust/TypeScript views. Handwritten parallel enums and registration lists are
drift bugs. Resource identity uses full generational keys, normally transported
as two `u32` values, so a stale handle cannot target a reused slot.
