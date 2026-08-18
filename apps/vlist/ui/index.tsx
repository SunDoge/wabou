// Virtual-list UI demo — 10,000 rows, only the viewport slice is materialised.
// Scrolling recycles slots via SolidJS `<Index>` (no create/drop churn), and
// the host clips with `overflow: hidden`. Pure SolidJS-as-DSL: no Rust widget,
// no native scroll container.

import "@wabou/core";
import "virtual:wabou-stylesheet";
import { createFps, mount, VirtualList } from "@wabou/core";
import { Text, View } from "@wabou/primitives";

const ROWS: readonly string[] = Array.from({ length: 10_000 }, (_, i) => {
  const tag = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta"][
    i % 7
  ];
  return `Row ${i} — ${tag} variant ${Math.floor(i / 7)}`;
});

function App() {
  const fps = createFps();
  return (
    <View class="w-full h-full flex flex-col bg-slate-950 text-slate-100">
      <View class="flex-none px-4 py-2 flex items-center justify-between border-b border-slate-700">
        <Text class="text-base font-semibold">Virtual list — 10,000 rows</Text>
        <Text class="text-xs font-mono text-slate-400">
          {ROWS.length} items · {fps()} fps
        </Text>
      </View>
      <View class="flex-1 min-h-0">
        <VirtualList
          items={() => ROWS}
          itemHeight={32}
          viewportHeight={540}
          overscan={6}
          role="listbox"
          accessibilityLabel="Virtual rows"
        >
          {(text, i) => (
            <View
              role="option"
              aria-label={text}
              class={`flex items-center h-full px-4 ${
                i % 2 ? "bg-slate-900" : "bg-slate-800/60"
              }`}
            >
              <Text class="w-20 flex-none text-slate-500 font-mono text-xs">
                {String(i).padStart(5, "0")}
              </Text>
              <Text class="overflow-hidden whitespace-nowrap text-sm">
                {text}
              </Text>
            </View>
          )}
        </VirtualList>
      </View>
    </View>
  );
}

mount(() => <App />);
