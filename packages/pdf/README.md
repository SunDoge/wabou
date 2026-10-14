# @wabou/pdf

Typed Solid bindings for Wabou's optional pure-Rust PDF page renderer.

Register the Rust widget when constructing the host:

```rust
let host = HostBuilder::new()
    .plugin(wabou_pdf::plugin());
```

Then use the typed Solid component. Page numbers are one-based:

```tsx
import { PdfView } from "@wabou/pdf";

<PdfView
  path="/home/me/Documents/book.pdf"
  page={currentPage()}
  class="h-full w-full bg-zinc-900"
/>
```

Parsing and rasterization run outside the UI thread. While `path`, `page`, or
`renderScale` changes, the widget retains the previous page until the latest
request finishes. `renderScale` defaults to 2; raising it improves zoomed text
at the cost of roughly quadratic pixel memory.

`PdfView`, its props, the native tag, and the config field split are generated
from `PdfViewerConfig` in the Rust crate. Regenerate or verify them with:

```bash
cargo run -p wabou-pdf --features bindings \
  --example wabou-pdf-bindings -- write
cargo run -p wabou-pdf --features bindings \
  --example wabou-pdf-bindings -- check
```
