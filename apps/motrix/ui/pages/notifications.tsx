import {
  Badge,
  Button,
  Card,
  CardContent,
  Empty,
  EmptyDescription,
  EmptyMedia,
  EmptyTitle,
  Icon,
  PageHeader,
  ScrollArea,
  Text,
  useNavigate,
  View,
} from "@wabou/ui";
import { Button as PrimitiveButton } from "@wabou/ui/primitives";
import checkCircle from "lucide-static/icons/circle-check.svg?raw";
import xCircle from "lucide-static/icons/circle-x.svg?raw";
import { createMemo, createSignal, For as ForValue, Show } from "solid-js";
import { useDownloads } from "../downloads";

export function NotificationsPage() {
  const downloads = useDownloads();
  const navigate = useNavigate();
  const [unavailableTask, setUnavailableTask] = createSignal("");
  const recentEvents = createMemo(() => downloads.events());
  return (
    <View class="h-full min-h-0 flex flex-col gap-4">
      <PageHeader
        title="Notifications"
        titleAdornment={
          <Badge variant="outline">{downloads.events().length}</Badge>
        }
        actions={
          <Button
            variant="ghost"
            disabled={!downloads.events().length}
            onClick={downloads.clearEvents}
          >
            Clear all
          </Button>
        }
      />
      <Show when={unavailableTask()}>
        <View
          role="alert"
          aria-label={`Task unavailable: ${unavailableTask()}`}
          class="px-4 py-3 rounded-lg bg-danger-surface"
        >
          <Text class="text-sm text-danger-primary">
            This task is no longer available: {unavailableTask()}
          </Text>
        </View>
      </Show>
      <Card class="min-h-0 flex-1 rounded-2xl shadow-md">
        <CardContent class="h-full min-h-0 p-0">
          <Show
            when={recentEvents().length}
            fallback={
              <Empty class="h-full min-h-0 rounded-none border-0 shadow-none">
                <EmptyMedia variant="icon" class="rounded-full">
                  <Icon source={checkCircle} size={22} class="text-muted" />
                </EmptyMedia>
                <EmptyTitle>No recent notifications</EmptyTitle>
                <EmptyDescription>
                  Completed and failed tasks will appear here.
                </EmptyDescription>
              </Empty>
            }
          >
            <View
              role="group"
              aria-label="Notification history"
              class="h-full min-h-0"
            >
              <ScrollArea class="h-full min-h-0" contentClass="flex flex-col">
                <ForValue each={recentEvents()}>
                  {(event) => {
                    const title =
                      event.status === "complete"
                        ? "Download complete"
                        : "Download failed";
                    return (
                      <PrimitiveButton
                        unstyled
                        aria-label={`View ${title}: ${event.name}`}
                        class={(state) =>
                          `w-full min-h-16 flex-none px-5 py-3 flex items-center gap-3 border-b border-subtle ${state.hovered ? "bg-control-hover" : "bg-transparent"} ${state.focusVisible ? "border-focus" : ""}`
                        }
                        onClick={async () => {
                          setUnavailableTask("");
                          if (!downloads.requestTaskInspection(event.taskId)) {
                            setUnavailableTask(event.name);
                            return;
                          }
                          await navigate({ to: "/downloads" });
                        }}
                      >
                        <View
                          class={`w-10 h-10 flex-none rounded-full flex items-center justify-center ${event.status === "complete" ? "bg-success-surface" : "bg-danger-surface"}`}
                        >
                          <Icon
                            source={
                              event.status === "complete"
                                ? checkCircle
                                : xCircle
                            }
                            size={19}
                            class={
                              event.status === "complete"
                                ? "text-success-primary"
                                : "text-danger-primary"
                            }
                          />
                        </View>
                        <View class="min-w-0 flex-1 flex flex-col gap-0.5">
                          <Text class="font-medium">{title}</Text>
                          <Text class="truncate text-sm text-muted">
                            {event.name}
                          </Text>
                        </View>
                        <Text class="flex-none text-xs text-muted">
                          {event.time}
                        </Text>
                      </PrimitiveButton>
                    );
                  }}
                </ForValue>
              </ScrollArea>
            </View>
          </Show>
        </CardContent>
      </Card>
    </View>
  );
}
