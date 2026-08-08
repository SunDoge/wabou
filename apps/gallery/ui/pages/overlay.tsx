import { Button } from "@wabou/components";
import { Text, View } from "@wabou/primitives";
import { Portal } from "@wabou/solid-renderer";
import { createSignal, Show } from "solid-js";

import { Preview } from "../preview";

export function OverlayPage() {
  const [open, setOpen] = createSignal(false);
  return (
    <Preview title="Modal plane and semantic isolation">
      <View class="p-4 flex items-start">
        <Button onClick={() => setOpen(true)}>Open modal overlay</Button>
      </View>
      <Show when={open()}>
        <Portal
          plane="modal"
          role="dialog"
          aria-label="Overlay settings"
          class="absolute left-0 top-0 w-full h-full flex items-center justify-center bg-slate-950"
          onClick={() => setOpen(false)}
          onKeyDown={(event: { key?: string }) => {
            if (event.key === "Escape") setOpen(false);
          }}
        >
          <View
            class="w-96 p-6 flex flex-col gap-4 rounded-xl border border-slate-700 bg-slate-900"
            onClick={(event: { stopPropagation(): void }) =>
              event.stopPropagation()
            }
          >
            <Text class="text-xl font-semibold text-white">Modal overlay</Text>
            <Text class="text-sm text-slate-300">
              {
                "This subtree is painted and hit-tested above floating content. While open, AccessKit exposes only the topmost modal beneath the window."
              }
            </Text>
            <View class="flex justify-end">
              <Button onClick={() => setOpen(false)}>Close</Button>
            </View>
          </View>
        </Portal>
      </Show>
    </Preview>
  );
}
