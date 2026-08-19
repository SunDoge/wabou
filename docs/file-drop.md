# Native file drag and drop

Wabou exposes window-level native file drops without a browser `DataTransfer`
compatibility layer. Import the API from the application-facing package:

```tsx
import { useFileDrop } from "@wabou/ui";

function App() {
  useFileDrop((event) => {
    if (event.phase === "dropped") {
      for (const path of event.paths) openFile(path);
    }
  });

  return <View>{/* application UI */}</View>;
}
```

Events have four explicit phases: `entered`, `moved`, `left`, and `dropped`.
`entered` and `dropped` include all paths supplied by the platform. `position`
uses logical window pixels and can be `null` when the platform does not report
it. `useFileDrop` automatically unsubscribes with its Solid owner; use
`subscribeFileDrop` outside a Solid owner and call the returned cleanup
function yourself.

The first version intentionally targets the whole native window. A component
can compare `event.position` with measured layout when it needs a drop zone;
node-level bubbling can be added later without pretending native paths are web
files.
