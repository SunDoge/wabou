// Virtual-list UI demo — 10,000 retained rows, with GPUI materializing only
// the visible range for native layout, paint, scrolling and clipping.

import "virtual:wabou-stylesheet";
import { Button, createFps, mount, Text, View, VirtualList } from "@wabou/ui";
import { createMemo, createSignal } from "solid-js";

const ROWS: readonly string[] = Array.from({ length: 10_000 }, (_, i) => {
  const tag = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta"][
    i % 7
  ];
  return `Row ${i} — ${tag} variant ${Math.floor(i / 7)}`;
});

function App() {
  const fps = createFps();
  const [condensed, setCondensed] = createSignal(false);
  const rows = createMemo(() => (condensed() ? ROWS.slice(0, 24) : ROWS));
  return (
    <View class="w-full h-full flex flex-col bg-slate-950 text-slate-100">
      <View class="flex-none px-4 py-2 flex items-center justify-between border-b border-slate-700">
        <Text class="text-base font-semibold">Virtual list — 10,000 rows</Text>
        <View class="flex items-center gap-3">
          <Text class="text-xs font-mono text-slate-400">
            {rows().length} items · {fps()} fps
          </Text>
          <Button
            size="sm"
            variant="outline"
            onClick={() => setCondensed((value) => !value)}
          >
            {condensed() ? "Show all rows" : "Show 24 rows"}
          </Button>
        </View>
      </View>
      <View class="flex-1 min-h-0">
        <VirtualList
          items={rows}
          getItemKey={(_, index) => index}
          itemHeight={32}
          viewportHeight={540}
          role="listbox"
          accessibilityLabel="Virtual rows"
        >
          {(text, i) => (
            <View
              role="option"
              aria-label={text()}
              class={`flex items-center h-full px-4 ${
                i() % 2 ? "bg-slate-900" : "bg-slate-800/60"
              }`}
            >
              <Text class="w-20 flex-none text-slate-500 font-mono text-xs">
                {String(i()).padStart(5, "0")}
              </Text>
              <Text class="overflow-hidden whitespace-nowrap text-sm">
                {text()}
              </Text>
            </View>
          )}
        </VirtualList>
      </View>
    </View>
  );
}

mount(() => <App />);
