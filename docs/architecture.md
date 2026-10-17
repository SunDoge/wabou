# Architecture boundaries

Wabou is a Rust application runtime with replaceable UI-language adapters. The
first adapter runs Solid 2 inside QuickJS and emits a versioned mutation
protocol into a retained native runtime. The current backend uses Winit for
windows and input, Taffy for layout, Parley for text, and Vello GPU for
painting.

```text
Rust application state and resources
                 |
        typed commands and events
                 |
       UI-language adapter
       (Solid 2 + QuickJS today)
                 |
          completed adapter flush
                 |
     batched, versioned operations
                 |
        retained Rust projection
       /          |          \
    Taffy       Parley    native widgets
       \          |          /
          Vello GPU scene
                 |
               Winit
```

## Sources of truth

Wabou keeps one authoritative owner for each kind of state:

- Rust owns durable application state, resources, asynchronous work, database
  connections, and operating-system integration.
- The active UI adapter owns authored component structure and disposable UI
  state such as local disclosure or hover state. Durable form drafts and other
  state that must survive a frontend reload belong in Rust.
- `crates/wabou-protocol/src/wire.rs` owns the binary wire format. TypeScript
  constants are generated from those Rust definitions; golden frames verify
  the encoder and decoder against the same numeric contract.
- The committed Rust projection tree owns the latest native node structure. A
  partially decoded frame is never observable.
- Taffy owns completed layout. The shell owns clipping, hit testing, focus,
  window integration, and accessibility publication.
- Native `TextInput` and `TextArea` widgets own their small text documents. The
  general `Editor` instead uses a DOM-free CodeMirror document as its source of
  truth and projects selection, IME composition, and syntax ranges into a
  controlled native viewport.
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

Rust owns application state and behavior. The UI adapter owns component
composition, routing presentation, interaction policy, and semantic intent. A
primitive declares capabilities such as focus participation explicitly. Rust
does not infer web behavior from tag names, `href`, or CSS conventions.

Rust validates and executes that intent: retained nodes, resources, layout,
text, paint, native input, windows, and operating-system services. Inference is
limited to local native work such as intrinsic measurement or deriving an
accessible label from explicit descendants.

Native widgets follow the same split. The UI adapter authors a complete configuration
snapshot; the widget measures, paints, and handles transient input; typed
events return facts to the adapter. See [native widgets](native-widgets.md).

## Public surfaces

- `@wabou/ui` is the first application-facing JSX adapter and component API.
- `@wabou/core` contains the embedded renderer and protocol runtime.
- `@wabou/vite` owns build integration and static style compilation.
- `@wabou/test` owns component, layout, native behavior, and capture tooling.
- `@wabou/terminal` is the optional terminal component package.
- `wabou` is the Rust application runtime and the application-facing source of
  truth for commands, events, resources, and native extensions. Applications
  normally use `wabou::HostBuilder` and do not depend on backend internals.

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
| generated bindings | typed Rust commands, events, data, and resource handles |

QuickJS values are an implementation detail of the current adapter. Public Rust
contracts use a restricted, language-neutral type vocabulary. Binding backends
derive TypeScript declarations today and can derive Dart or another frontend
language later without changing application services. Long-running native
producers publish typed events; resource identity crosses the adapter as an
opaque generational handle.

The mutation protocol is likewise independent of Solid and QuickJS. A future
adapter needs to produce the same retained-tree operations, receive typed
events, and implement the generated command surface. It does not reimplement
layout, text, rendering, resource ownership, or application business logic.

New cross-language features need one authoritative Rust declaration and a
generated view for each adapter. Handwritten parallel enums and registration
lists are drift bugs. Resource identity uses full generational keys, normally
transported as two `u32` values, so a stale handle cannot target a reused slot.
