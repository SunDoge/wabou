import {
  Badge,
  Button,
  Card,
  CardContent,
  Ripple,
  Text,
  View,
} from "@wabou/ui";
import { For, Show } from "solid-js";
import { useAria2 } from "../aria2";
import { LiveChart } from "../components/live-chart";
import { StatCard } from "../components/stat-card";
import { formatBytes } from "../lib/format";

export function DashboardPage() {
  const aria2 = useAria2();
  const snapshot = aria2.snapshot;
  const speedLimit = () => aria2.config().maxOverallDownloadLimit;
  const setSpeedLimit = async (limit: string) => {
    await aria2.saveConfig({
      ...aria2.config(),
      maxOverallDownloadLimit: limit,
    });
  };
  return (
    <View class="h-full flex flex-col gap-5">
      <Text role="heading" class="text-4xl font-bold">
        Dashboard
      </Text>
      <View class="h-52 flex flex-none gap-5">
        <StatCard
          label="ENGINE"
          value={snapshot().connected ? "Ready" : "Offline"}
          detail={
            snapshot().version
              ? `aria2 v${snapshot().version}`
              : snapshot().endpoint
          }
        >
          <Show
            when={snapshot().connected}
            fallback={
              <Badge variant="secondary">
                <View class="w-2 h-2 rounded-full bg-muted" />
                {snapshot().error === "Connecting to aria2…"
                  ? "Connecting"
                  : "Disconnected"}
              </Badge>
            }
          >
            <Badge variant="success">
              <View class="relative w-3 h-3 flex items-center justify-center">
                <Ripple
                  aria-hidden
                  duration={1.4}
                  class="absolute inset-0 rounded-full bg-success-primary"
                />
                <View class="w-2 h-2 rounded-full bg-success-primary" />
              </View>
              Connected
            </Badge>
          </Show>
        </StatCard>
        <StatCard
          label="SPEED LIMIT"
          value={speedLimit() === "0" ? "Full" : speedLimit()}
          detail="Global download limit"
        >
          <View class="flex gap-2">
            <Button
              size="sm"
              variant={speedLimit() === "0" ? "default" : "outline"}
              onClick={() => setSpeedLimit("0")}
            >
              Full
            </Button>
            <Button
              size="sm"
              variant={speedLimit() === "10M" ? "default" : "outline"}
              onClick={() => setSpeedLimit("10M")}
            >
              10M
            </Button>
            <Button
              size="sm"
              variant={speedLimit() === "1M" ? "default" : "outline"}
              onClick={() => setSpeedLimit("1M")}
            >
              1M
            </Button>
          </View>
        </StatCard>
        <StatCard
          label="UPLOAD"
          value={`${formatBytes(snapshot().uploadSpeed)}/s`}
          detail="Live RPC value"
        />
        <StatCard
          label="DOWNLOAD"
          value={`${formatBytes(snapshot().downloadSpeed)}/s`}
          detail="Live RPC value"
        >
          <LiveChart values={aria2.downloadHistory()} />
        </StatCard>
      </View>
      <View class="h-32 flex flex-none gap-5">
        <StatCard
          label="ACTIVE TASKS"
          value={String(
            snapshot().tasks.filter((task) => task.status === "active").length,
          )}
          detail="Reported by aria2"
        />
        <StatCard
          label="TRANSFER TODAY"
          value={formatBytes(snapshot().downloadedToday)}
          detail="Persisted for the current local day"
        />
      </View>
      <Card class="min-h-0 flex-1 rounded-2xl shadow-xl">
        <CardContent class="h-full p-5 flex flex-col gap-3">
          <Text class="text-xs font-medium text-muted">ACTIVITY</Text>
          <View class="min-h-0 flex-1 grid grid-cols-12 gap-2">
            <For each={snapshot().activity}>
              {(value) => (
                <View
                  class={
                    value > 0
                      ? "h-4 rounded-sm bg-accent"
                      : "h-4 rounded-sm bg-control"
                  }
                />
              )}
            </For>
          </View>
        </CardContent>
      </Card>
    </View>
  );
}
