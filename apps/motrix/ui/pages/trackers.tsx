import {
  Badge,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Empty,
  EmptyDescription,
  EmptyMedia,
  EmptyTitle,
  Icon,
  PageHeader,
  ResponsiveGrid,
  ScrollArea,
  Text,
  View,
} from "@wabou/ui";
import activity from "lucide-static/icons/activity.svg?raw";
import network from "lucide-static/icons/network.svg?raw";
import radioTower from "lucide-static/icons/radio-tower.svg?raw";
import route from "lucide-static/icons/route.svg?raw";
import users from "lucide-static/icons/users.svg?raw";
import { createMemo, For, Show } from "solid-js";
import type { DownloadTask } from "../downloads";
import { useDownloads } from "../downloads";

export interface TrackerEndpoint {
  url: string;
  protocol: string;
  tasks: number;
}

export function trackerEndpoints(
  tasks: readonly DownloadTask[],
): TrackerEndpoint[] {
  const endpoints = new Map<string, Set<string>>();
  for (const task of tasks) {
    if (!task.bittorrent || !task.uri?.startsWith("magnet:?")) continue;
    const query = task.uri.slice(task.uri.indexOf("?") + 1);
    for (const item of query.split("&")) {
      const [name, encoded = ""] = item.split("=", 2);
      if (name !== "tr") continue;
      let url: string;
      try {
        url = decodeURIComponent(encoded.replaceAll("+", "%20")).trim();
      } catch {
        continue;
      }
      if (!url) continue;
      const taskIds = endpoints.get(url) ?? new Set<string>();
      taskIds.add(task.id);
      endpoints.set(url, taskIds);
    }
  }
  return [...endpoints]
    .map(([url, taskIds]) => ({
      url,
      protocol: url.slice(0, url.indexOf(":")) || "unknown",
      tasks: taskIds.size,
    }))
    .sort((left, right) => left.url.localeCompare(right.url));
}

function DiscoveryCard(props: {
  label: string;
  value: string;
  detail: string;
  icon: string;
}) {
  return (
    <Card
      role="group"
      aria-label={`${props.label} discovery status`}
      class="min-w-0 rounded-2xl shadow-sm"
    >
      <CardContent class="p-5 flex-row items-center gap-4">
        <View class="w-11 h-11 flex-none rounded-xl bg-selected flex items-center justify-center">
          <Icon source={props.icon} size={20} class="text-accent" />
        </View>
        <View class="min-w-0 flex-1 flex flex-col gap-1">
          <Text class="text-xs font-medium text-muted">{props.label}</Text>
          <Text class="truncate text-lg font-semibold">{props.value}</Text>
          <Text class="truncate text-xs text-muted">{props.detail}</Text>
        </View>
      </CardContent>
    </Card>
  );
}

export function TrackersPage() {
  const downloads = useDownloads();
  const torrentTasks = createMemo(() =>
    downloads.snapshot().tasks.filter((task) => task.bittorrent),
  );
  const endpoints = createMemo(() => trackerEndpoints(torrentTasks()));
  const connections = createMemo(() =>
    torrentTasks().reduce((total, task) => total + task.connections, 0),
  );
  const config = () => downloads.config();
  const nat = () => downloads.snapshot().nat;

  return (
    <View class="h-full min-h-0 flex flex-col gap-4">
      <PageHeader
        title="Trackers"
        description="BitTorrent tracker and peer discovery overview"
        titleAdornment={<Badge variant="outline">{endpoints().length}</Badge>}
      />

      <ResponsiveGrid
        minColumnWidth={220}
        gap={12}
        maxColumns={2}
        initialColumns={2}
      >
        <DiscoveryCard
          label="DHT DISCOVERY"
          value={config().dhtEnabled ? "Enabled" : "Disabled"}
          detail="Distributed trackerless peer lookup"
          icon={radioTower}
        />
        <DiscoveryCard
          label="PEER EXCHANGE"
          value={config().pexEnabled ? "Enabled" : "Disabled"}
          detail={`${connections()} active peer connections`}
          icon={users}
        />
        <DiscoveryCard
          label="LISTEN PORT"
          value={String(config().listenPort)}
          detail={`${torrentTasks().length} BitTorrent tasks`}
          icon={network}
        />
        <DiscoveryCard
          label="NAT MAPPING"
          value={nat().enabled ? nat().state : "Disabled"}
          detail={nat().tcpExternalAddress ?? "No external address reported"}
          icon={route}
        />
      </ResponsiveGrid>

      <Card
        role="group"
        aria-label="Tracker endpoints"
        class="min-h-0 flex-1 rounded-2xl shadow-md"
      >
        <CardHeader class="flex-row items-start justify-between gap-3">
          <View class="min-w-0 flex-1 flex flex-col gap-1">
            <CardTitle>Tracker endpoints</CardTitle>
            <CardDescription>
              Trackers declared by currently loaded magnet tasks
            </CardDescription>
          </View>
          <Badge variant="secondary">{torrentTasks().length} tasks</Badge>
        </CardHeader>
        <CardContent class="min-h-0 flex-1 p-0">
          <Show
            when={endpoints().length > 0}
            fallback={
              <Empty
                role="status"
                aria-label="No tracker endpoints yet"
                class="h-full min-h-56 rounded-none border-0 shadow-none"
              >
                <EmptyMedia variant="icon" class="rounded-full">
                  <Icon source={activity} size={22} class="text-muted" />
                </EmptyMedia>
                <EmptyTitle>No tracker endpoints yet</EmptyTitle>
                <EmptyDescription>
                  Add a magnet link with tracker parameters to inspect its
                  discovery endpoints here. Trackerless tasks can still use DHT
                  and peer exchange.
                </EmptyDescription>
              </Empty>
            }
          >
            <ScrollArea class="h-full min-h-0" contentClass="flex flex-col">
              <For each={endpoints()}>
                {(endpoint) => (
                  <View class="min-h-16 flex-none px-5 py-3 flex items-center gap-3 border-t border-subtle">
                    <View class="w-9 h-9 flex-none rounded-lg bg-control flex items-center justify-center">
                      <Icon source={radioTower} size={17} class="text-muted" />
                    </View>
                    <View class="min-w-0 flex-1 flex flex-col gap-1">
                      <Text class="truncate text-sm font-medium">
                        {endpoint.url}
                      </Text>
                      <Text class="text-xs text-muted">
                        Used by {endpoint.tasks}{" "}
                        {endpoint.tasks === 1 ? "task" : "tasks"}
                      </Text>
                    </View>
                    <Badge variant="outline">
                      {endpoint.protocol.toUpperCase()}
                    </Badge>
                  </View>
                )}
              </For>
            </ScrollArea>
          </Show>
        </CardContent>
      </Card>
    </View>
  );
}
