// Virtual-list UI demo — 10,000 rows, only the viewport slice is materialised.
// Scrolling recycles slots via SolidJS `<Index>` (no create/drop churn), and
// the host clips with `overflow: hidden`. Pure SolidJS-as-DSL: no Rust widget,
// no native scroll container.

import "@wabou/core";
import "virtual:wabou-stylesheet";
import { createFps, mount, VirtualList } from "@wabou/solid-renderer";

const ROWS: readonly string[] = Array.from({ length: 10_000 }, (_, i) => {
  const tag = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta"][
    i % 7
  ];
  return `Row ${i} — ${tag} variant ${Math.floor(i / 7)}`;
});

function App() {
  const fps = createFps();
  return (
    <div class="w-full h-full flex flex-col bg-slate-950 text-slate-100">
      <div class="flex-none px-4 py-2 flex items-center justify-between border-b border-slate-700">
        <span class="text-base font-semibold">Virtual list — 10,000 rows</span>
        <span class="text-xs font-mono text-slate-400">
          {ROWS.length} items · {fps()} fps
        </span>
      </div>
      <div class="flex-1 min-h-0">
        <VirtualList
          items={() => ROWS}
          itemHeight={32}
          viewportHeight={540}
          overscan={6}
        >
          {(text, i) => (
            <div
              class={`flex items-center h-full px-4 ${
                i % 2 ? "bg-slate-900" : "bg-slate-800/60"
              }`}
            >
              <span class="w-20 flex-none text-slate-500 font-mono text-xs">
                {String(i).padStart(5, "0")}
              </span>
              <span class="overflow-hidden whitespace-nowrap text-sm">
                {text}
              </span>
            </div>
          )}
        </VirtualList>
      </div>
    </div>
  );
}

mount(() => <App />);
