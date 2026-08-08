# Wabou

[![CI](https://github.com/SunDoge/wabou/actions/workflows/ci.yml/badge.svg)](https://github.com/SunDoge/wabou/actions/workflows/ci.yml)

Wabou is an experimental native UI runtime for building desktop applications with SolidJS and Rust. It runs application logic in QuickJS, then performs layout, painting, and native integration in Rust—without embedding a browser or a WebView.

> Wabou is under active development. The architecture is usable for experimentation, but the public API and platform support are not stable yet.

## Showcase

### Component Gallery

An interactive catalogue of Wabou components and platform features, including
animations, native windows, and Rust-powered custom widgets.

<p align="center">
<img width="45%" alt="image" src="https://github.com/user-attachments/assets/b0863daf-7255-443e-b8df-d54eb4074c43" />
<img width="45%" alt="image" src="https://github.com/user-attachments/assets/85b7ac8d-eade-46d5-9022-4c46f9f99509" />
</p>

### Terminal

A native terminal widget powered by [rio-vt](https://crates.io/crates/rio-vt), demonstrating keyboard and pointer input, text
selection, clipboard integration, scrolling, and system font discovery.

<img width="1068" height="776" alt="image" src="https://github.com/user-attachments/assets/8575bd24-30f2-42d0-be9f-f456b124b998" />


## Why Wabou?

Web UI is productive, but a browser engine is a large runtime to ship when an application only needs a reactive component model. Traditional native UI can be lean and fast, but often gives up the component ergonomics and iteration speed frontend developers expect.

Wabou explores a narrower combination:

- SolidJS signals and JSX for application state and composition;
- QuickJS as a small, embeddable JavaScript runtime;
- a compact binary protocol instead of a DOM or JSON bridge;
- retained layout with Taffy and GPU rendering with Vello;
- Rust for windows, input, text, native widgets, and platform integration;
- Vite HMR and an inspector for a short development loop.

## Why SolidJS?

Solid fits an embedded runtime particularly well because its reactivity is
fine-grained. A signal update runs the computations that depend on it; it does
not require rebuilding and diffing a virtual component tree every frame. That
keeps both JavaScript work and bridge traffic proportional to what changed.

Wabou implements Solid's universal renderer rather than emulating a browser
DOM. Solid owns signals, effects, component lifetimes, and reconciliation, while
the Wabou host emits only the mutations needed by the retained Rust tree. This
also lets state stay explicit in Solid primitives: styling does not create a
second, implicit state machine through CSS pseudo-classes.

The result is a familiar declarative model with a deliberately small boundary
between JavaScript and native code.

```text
Solid signals + JSX
        │ changed nodes only
        ▼
Solid universal renderer
        │ compact binary operations
        ▼
QuickJS ───────────────► Rust host ──► Taffy layout ──► Vello / native widgets
   ▲                         │
   └──── input, timers, and host events ───────────────┘
```

## How 60/120 fps works on QuickJS

Wabou does not make QuickJS render pixels, and “120 fps” is not an unconditional
benchmark claim. The display refresh rate supplies the cadence; Wabou arranges
the work so JavaScript does not have to be the whole frame.

For every active display frame:

1. The Rust host delivers pending input and host messages.
2. It runs one `requestAnimationFrame` turn in QuickJS with the frame timestamp.
3. Solid updates enqueue binary mutations, which are flushed once at the end of
   that turn.
4. Rust applies the mutations to its retained tree, recomputes only invalidated
   work, and lays out and paints at vsync.

On a 60 Hz display, all frame work must fit in roughly **16.67 ms**. On a 120 Hz
display, it must fit in roughly **8.33 ms**. Solid's targeted updates, batched
binary transport, retained native state, and Rust-side layout/rendering are how
Wabou makes those budgets attainable. Application code can still miss them:
long JavaScript callbacks, excessive layout invalidation, expensive painting,
or too many visible nodes will drop frames. Large collections should be
windowed, and animation work should stay small and measurable.

Continuous redraw is demand-driven. A queued `requestAnimationFrame` keeps the
host drawing at the display cadence; when no animation is active, Wabou returns
to event-driven rendering instead of spending CPU producing identical frames.
Async jobs also have a bounded scheduler budget so an unbounded microtask chain
cannot monopolize a UI callback.

Use the [DevTools inspector](docs/devtools.md) to observe QuickJS tick, frame
build, scene, and presentation timing. Performance claims should be accompanied
by a reproducible workload, hardware, operating system, scale factor, and
refresh rate.

## Try it

You need a current Rust toolchain and
[mise](https://mise.jdx.dev/). Clone the repository, then run:

```bash
mise install
mise exec -- bun install
mise exec -- bun run wabou run --app-dir apps/gallery
```

Other examples include a [Hacker News client](apps/hackernews) and a
[terminal](apps/terminal). See the [CLI guide](docs/cli.md) for development and
packaging commands.

## Design principles

- **Explicit state.** Interaction state belongs in Solid signals and props, not
  hidden CSS state machines.
- **Pay for changes.** Batch mutations and preserve layout and render state
  across frames.
- **Idle means idle.** Redraw continuously only while animation requests it.
- **Native where it matters.** Keep platform access and performance-sensitive
  work in Rust without making every UI change a Rust compile cycle.
- **Measure the real layer.** A successful build is not evidence of smooth
  rendering; verify frame timing and output on the target platform and display.

Read more about [styling](docs/style.md), [windows](docs/windows.md),
[accessibility](docs/accessibility.md),
[host-to-JavaScript communication](docs/host-to-js.md), and
[composable events](docs/composable-events.md).

## Contributing

Wabou is looking for contributors interested in native rendering, SolidJS
integration, developer tooling, accessibility, text, and cross-platform UI.
The most useful contributions today are focused bug reports with a reproduction,
small examples that stress a real application pattern, and improvements that
make the public API smaller and more predictable.

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, checks, and commit conventions.
If you are evaluating Wabou for an application, open an issue describing the
use case and the missing capability; real constraints are more valuable than a
generic feature wishlist.

## Acknowledgments & Inspiration

This project was deeply inspired by [PocketJS](https://github.com/pocket-stack/pocketjs). Their pioneering work in Rust-based minimalistic UI rendering and architectural design significantly influenced the direction of `wabou`.

## License

Wabou is licensed under the [Apache License 2.0](LICENSE).
