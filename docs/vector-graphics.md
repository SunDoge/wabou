# Vector graphics

Wabou exposes a small retained path API instead of emulating SVG DOM or the
Canvas 2D state machine. Application code builds an immutable `VectorPath`
snapshot and passes it to the `Path` primitive:

```tsx
import { Path, PathBuilder } from "@wabou/ui";

const curve = new PathBuilder()
  .moveTo(8, 48)
  .cubicTo(40, 4, 80, 92, 120, 24)
  .build({ stroke: 0x38bdf8ff, strokeWidth: 3, lineCap: "round" });

<Path class="w-32 h-16" source={curve} />;
```

The stable path vocabulary is `moveTo`, `lineTo`, `quadTo`, `cubicTo`, and
`close`. Paint currently includes solid fill and stroke colors, fill rule,
stroke width, caps, joins, and miter limit. Coordinates use local logical
pixels. Wabou does not implicitly scale geometry to the node bounds.

The UI protocol carries this through the generic
`SetGraphicData(kind, bytes)` operation. Each graphic kind owns its own
versioned, length-delimited payload; the Rust protocol layer routes opaque
bytes and the relevant renderer validates them atomically before replacing
retained state. This leaves room for other public graphic kinds without
adding a top-level opcode for every drawing command.

Libraries such as D3 can be adapted at the application boundary. Their
Canvas-shaped context is not part of Wabou's public contract: an adapter maps
the library's calls to `PathBuilder`, while layout, semantics, input, and
painting remain native Wabou behavior.
