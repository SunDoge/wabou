# Web runtime boundaries

Wabou keeps QuickJS and implements browser-independent APIs selectively. Inspired
by den's separation of native Web APIs, parsing and codec state live in Rust while
small JavaScript wrappers preserve object identity and JavaScript iteration.
Wabou does not depend on den.

The default runtime entry contains only the baseline needed by every
application. Applications that use `fetch`, `Response`, or WHATWG Streams must
import `@wabou/ui/web` once. Lower-level integrations can import
`@wabou/core/web` directly. This keeps the stream implementation out of UI-only
bundles while preserving the same globals for networked applications.

- URL parsing, setters, origins and form query encoding use the existing `url`
  crate. URLSearchParams remains the same object when its URL changes and mutations
  update the URL. The `whatwg-url` npm dependency is removed.
- Buffered fetch consumption reuses its single owned body chunk; arrayBuffer
  also reuses an exact-sized backing buffer. User-supplied input and clones stay
  isolated, and subviews are copied when needed to hide unrelated bytes. The
  Response releases its cached clone source when a reader takes ownership.
- TextDecoder uses a Rust-owned `encoding_rs` decoder, including encoding labels,
  streaming, fatal errors, and BOM handling. Synchronous decoding borrows the
  input without an intermediate Rust Vec. It retains no Ctx or JS handles.
- TextEncoder.encodeInto writes into a bounded destination in one native call,
  counts UTF-16 input units and never writes a partial encoded scalar.
- TextEncoderStream and TextDecoderStream compose TransformStream with the native
  codecs. Split surrogate pairs, incomplete input, fatal errors, and cancellation
  are covered without the `@stardazed/streams-text-encoding` dependency.
- AbortController retains the existing event polyfill; reason, throwIfAborted,
  abort, timeout and any are added. Fetch cancellation uses runtime-owned Rust
  cancellation tokens, dropping the reqwest future and its connection. Request
  tokens are allocated before async execution so cancellation before polling works.
- Native builds use Wabou's small native/Jiff localization API. Browser previews
  delegate the same typed API to the browser's built-in `Intl` implementation.

These contracts are exercised inside QuickJS, including teardown with live codec
objects, and real local HTTP tests verify both cancellation before connection and
closure of an in-flight connection. This is not a claim of full browser/WPT
conformance: fetch still buffers response bodies and supports a limited request
surface, URL blob object URLs are not provided, and Streams remain polyfilled.

The embedded prelude dropped from 486,851 bytes to approximately 15 KB in this
change. This is uncompressed embedded JavaScript, not total application size;
encoding_rs also adds Rust code. No total distribution or startup improvement is
claimed without measurement.

References:

- https://github.com/stevefan1999-personal/den/blob/master/den-stdlib-whatwg/src/url.rs
- https://github.com/stevefan1999-personal/den/blob/master/den-stdlib-text/src/lib.rs
