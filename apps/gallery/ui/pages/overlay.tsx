import { Button } from "@wabou/components";
import {
  createNotifications,
  Modal,
  NotificationRegion,
  Text,
  View,
} from "@wabou/primitives";
import { createSignal } from "solid-js";

import { Preview } from "../preview";

export function OverlayPage() {
  const [open, setOpen] = createSignal(false);
  const notifications = createNotifications();
  return (
    <Preview title="Modal plane and semantic isolation">
      <View class="p-4 flex items-start gap-2">
        <Button onClick={() => setOpen(true)}>Open modal overlay</Button>
        <Button
          variant="outline"
          onClick={() =>
            notifications.show({
              "aria-label": "Vault synchronized",
              content: ({ dismiss }) => (
                <>
                  <View class="min-w-0 flex-1 flex flex-col gap-1">
                    <Text class="text-sm font-medium text-white">
                      Vault synchronized
                    </Text>
                    <Text class="text-xs text-slate-400">
                      Your encrypted items are up to date.
                    </Text>
                  </View>
                  <Button size="sm" variant="ghost" onClick={dismiss}>
                    Dismiss
                  </Button>
                </>
              ),
            })
          }
        >
          Show notification
        </Button>
      </View>
      <Modal
        aria-label="Overlay settings"
        open={open()}
        onOpenChange={setOpen}
        backdropClass="bg-slate-950"
        contentClass="w-96 p-6 flex flex-col gap-4 rounded-xl border border-slate-700 bg-slate-900"
      >
        {({ close }) => (
          <>
            <Text class="text-xl font-semibold text-white">Modal overlay</Text>
            <Text class="text-sm text-slate-300">
              {
                "This subtree is painted and hit-tested above floating content. While open, AccessKit exposes only the topmost modal beneath the window."
              }
            </Text>
            <View class="flex justify-end">
              <Button onClick={close}>Close</Button>
            </View>
          </>
        )}
      </Modal>
      <NotificationRegion
        notifications={notifications}
        itemClass="w-96 rounded-xl border border-slate-700 bg-slate-900 p-4 flex items-center gap-3"
      />
    </Preview>
  );
}
