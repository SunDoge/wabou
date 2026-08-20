import { Badge, Button, Card, CardContent, Icon, Text, View } from "@wabou/ui";
import checkCircle from "lucide-static/icons/circle-check.svg?raw";
import xCircle from "lucide-static/icons/circle-x.svg?raw";
import { For, Show } from "solid-js";
import { useDownloads } from "../downloads";

export function NotificationsPage() {
  const downloads = useDownloads();
  return (
    <View class="flex flex-col gap-4">
      <View class="flex items-center justify-between">
        <View class="flex items-center gap-3">
          <Text role="heading" class="text-2xl font-bold">
            Notifications
          </Text>
          <Badge variant="outline">{downloads.events().length}</Badge>
        </View>
        <Button
          variant="ghost"
          disabled={!downloads.events().length}
          onClick={downloads.clearEvents}
        >
          Clear all
        </Button>
      </View>
      <Card class="rounded-xl shadow-lg">
        <CardContent class="p-0">
          <For each={downloads.events()}>
            {(event) => (
              <View class="min-h-14 px-4 flex items-center gap-3 border-b border-subtle">
                <View
                  class={`w-9 h-9 rounded-full flex items-center justify-center ${event.status === "complete" ? "bg-success-surface" : "bg-danger-surface"}`}
                >
                  <Icon
                    source={event.status === "complete" ? checkCircle : xCircle}
                    size={18}
                    class={
                      event.status === "complete"
                        ? "text-success-primary"
                        : "text-danger-primary"
                    }
                  />
                </View>
                <View class="min-w-0 flex-1 flex flex-col">
                  <Text class="font-medium">
                    {event.status === "complete"
                      ? "Download complete"
                      : "Download failed"}
                  </Text>
                  <Text class="truncate text-sm text-muted">{event.name}</Text>
                </View>
                <Text class="text-xs text-muted">{event.time}</Text>
              </View>
            )}
          </For>
          <Show when={!downloads.events().length}>
            <View class="h-64 flex flex-col items-center justify-center gap-3">
              <Icon source={checkCircle} size={34} class="text-muted" />
              <Text class="font-medium">No recent notifications</Text>
              <Text class="text-sm text-muted">
                Completed and failed tasks will appear here.
              </Text>
            </View>
          </Show>
        </CardContent>
      </Card>
    </View>
  );
}
