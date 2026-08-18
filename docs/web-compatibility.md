# Web compatibility contract

Wabou is a Solid universal renderer hosted in QuickJS. It is not a browser and
does not aim to implement the DOM or arbitrary CSS. Compatibility is explicit:
an API not listed here is unsupported unless a package provides its own
platform-independent implementation.

## Supported contracts

| Surface | Contract |
| --- | --- |
| JavaScript | QuickJS ES modules after bundling; standard language and Promise behavior supported by the embedded engine. |
| Solid | `solid-js` reactive primitives and the `solid-js/universal` renderer contract. Components must not assume browser DOM nodes. |
| Host nodes | `View`, `Text`, `Image`, native form controls, registered native widgets, and documented intrinsic tags. Handles expose Wabou operations, not `HTMLElement`. |
| Text | `<Text>` is the stable text aggregation boundary. Adjacent JSX text nodes are not implicitly merged into one browser-style text node. |
| Events | Documented pointer, keyboard, focus, input, wheel and host events. `Tab`/`Shift+Tab` follow native focus order, honor `tabIndex`, remain inside the topmost modal, and run only when JavaScript or the focused widget does not consume the key. |
| Text input | Native input and textarea widgets support IME preedit, commit, surrounding-text deletion, and candidate-window positioning. Registered widgets receive the same IME lifecycle and may expose their own local caret rectangle. |
| Layout/style | Utilities and typed inline properties listed by the generated Wabou utility manifest. Unsupported candidates are build errors. |
| Async/runtime | `fetch`, timers, `requestAnimationFrame`, URL APIs, text encoding, resize observation and the typed Wabou Host APIs. |
| Navigation | `@wabou/router` memory history. There is no browser address bar or implicit `window.history`. |
| Portals | Wabou overlay planes through `Portal`; no browser document/body target. |

## Deliberate differences

- There is no general `window`, `document`, `HTMLElement`, DOM traversal,
  `MutationObserver`, browser selection API, or CSSOM.
- CSS selectors, pseudo-classes, media queries, transitions and arbitrary
  runtime-generated class strings are not accepted. Interaction state is
  explicit Solid state; CSS does not create an implicit state machine.
- Tag names do not imply browser user-agent styles or the complete HTML
  accessibility model.
- JSX exposes a finite native host-tag and property registry. Browser-only
  elements and attributes such as anchors, `href`, `title`, and generic
  `type="password"` inputs are type errors; use an explicit Wabou component or
  native capability instead.
- Layout measurement is a completed native-frame snapshot through `useHost()`;
  synchronous DOM layout reads are unavailable.
- Browser storage, cookies, service workers, WebGL and browser canvas APIs are
  not supplied. A native widget may provide an explicit alternative.

## Package compatibility rule

A package is compatible when it uses Solid reactivity and platform-independent
JavaScript, or when every required platform capability has a Wabou adapter.
Successful bundling alone is not compatibility evidence. Packages that inspect
the DOM, patch browser globals, or rely on browser navigation need an adapter or
must be replaced.

New compatibility claims require a public API test in TypeScript and, for Host
behavior, execution through the generated embedded QuickJS fixture. Platform
behavior such as clipboard, windows, IME, fonts, accessibility and HiDPI also
requires native platform evidence.

See the [tested JavaScript library list](javascript-libraries.md) for specific
packages, versions, adapters and known incompatibilities.
