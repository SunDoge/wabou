import {
  Badge,
  Card,
  CardContent,
  Ripple,
  Select,
  Text,
  View,
} from "@wabou/ui";
import { createMemo, For, Show } from "solid-js";
import { match, P } from "ts-pattern";
import { useAria2 } from "../aria2";
import { LiveChart } from "../components/live-chart";
import { StatCard } from "../components/stat-card";
import { formatBytes } from "../lib/format";

const DAYS_PER_WEEK = 7;

function activityClass(value: number, maximum: number) {
  const ratio = value / Math.max(1, maximum);
  return match({ value, ratio })
    .with({ value: P.when((current) => current <= 0) }, () => ({
      "bg-control": true,
    }))
    .with({ ratio: P.when((current) => current <= 0.25) }, () => ({
      "bg-activity-1": true,
    }))
    .with({ ratio: P.when((current) => current <= 0.5) }, () => ({
      "bg-activity-2": true,
    }))
    .with({ ratio: P.when((current) => current <= 0.75) }, () => ({
      "bg-activity-3": true,
    }))
    .otherwise(() => ({ "bg-activity-4": true }));
}

export function DashboardPage() {
  const aria2 = useAria2();
  const snapshot = aria2.snapshot;
  const speedLimit = () => aria2.config().maxOverallDownloadLimit;
  const uploadLimit = () => aria2.config().maxOverallUploadLimit;
  const activeSpeedProfile = createMemo(() => {
    const index = aria2
      .config()
      .speedProfiles.findIndex(
        (profile) =>
          profile.downloadLimit === speedLimit() &&
          profile.uploadLimit === uploadLimit(),
      );
    return index < 0 ? "" : String(index);
  });
  const activeTasks = createMemo(
    () => snapshot().tasks.filter((task) => task.status === "active").length,
  );
  const waitingTasks = createMemo(
    () => snapshot().tasks.filter((task) => task.status === "waiting").length,
  );
  const stoppedTasks = createMemo(
    () => snapshot().tasks.filter((task) => task.status === "complete").length,
  );
  const activityMaximum = createMemo(() => Math.max(1, ...snapshot().activity));
  const activityWeeks = createMemo(() => {
    const days = snapshot().activity;
    const weeks: number[][] = [];
    for (let offset = 0; offset < days.length; offset += DAYS_PER_WEEK) {
      weeks.push(days.slice(offset, offset + DAYS_PER_WEEK));
    }
    return weeks;
  });
  const activeDays = createMemo(
    () => snapshot().activity.filter((value) => value > 0).length,
  );
  const setSpeedProfile = async (value: string) => {
    const profile = aria2.config().speedProfiles[Number.parseInt(value, 10)];
    if (!profile) return;
    await aria2.saveConfig({
      ...aria2.config(),
      maxOverallDownloadLimit: profile.downloadLimit,
      maxOverallUploadLimit: profile.uploadLimit,
    });
  };

  return (
    <View class="h-full flex flex-col gap-4">
      <View class="flex-none flex flex-row items-end justify-between">
        <View class="flex flex-col gap-1">
          <Text role="heading" class="text-3xl font-bold">
            Dashboard
          </Text>
          <Text class="text-sm text-muted">
            A clear view of your downloads and transfer activity.
          </Text>
        </View>
        <Badge variant={snapshot().connected ? "success" : "secondary"}>
          <View class="relative w-3 h-3 flex items-center justify-center">
            <Show when={snapshot().connected}>
              <Ripple
                aria-hidden
                duration={1.4}
                class="absolute inset-0 rounded-full bg-success-primary"
              />
            </Show>
            <View
              class="w-2 h-2 rounded-full"
              classList={{
                "bg-success-primary": snapshot().connected,
                "bg-muted": !snapshot().connected,
              }}
            />
          </View>
          {snapshot().connected ? "Engine ready" : "Engine offline"}
        </Badge>
      </View>

      <View class="h-40 flex flex-none gap-4">
        <StatCard
          label="ENGINE"
          accent={snapshot().connected ? "green" : "neutral"}
          value={snapshot().connected ? "Ready" : "Offline"}
          detail={
            snapshot().version
              ? `aria2 v${snapshot().version}`
              : snapshot().endpoint
          }
        >
          <Text class="text-xs text-secondary">
            {snapshot().connected
              ? "RPC is responding normally"
              : snapshot().error === "Connecting to aria2…"
                ? "Connecting to managed engine"
                : "Waiting for RPC connection"}
          </Text>
        </StatCard>
        <StatCard
          label="SPEED LIMIT"
          accent="neutral"
          value={speedLimit() === "0" ? "Unlimited" : speedLimit()}
          detail={`Upload ${uploadLimit() === "0" ? "unlimited" : uploadLimit()}`}
        >
          <Select
            aria-label="Speed profile"
            class="w-full"
            placeholder="Custom limits"
            value={activeSpeedProfile()}
            options={aria2.config().speedProfiles.map((profile, index) => ({
              value: String(index),
              label: profile.name,
            }))}
            onValueChange={(value) => void setSpeedProfile(value)}
          />
        </StatCard>
        <StatCard
          label="UPLOAD"
          accent="purple"
          value={`${formatBytes(snapshot().uploadSpeed)}/s`}
          detail="Live RPC throughput"
        >
          <LiveChart compact color="upload" values={aria2.uploadHistory()} />
        </StatCard>
        <StatCard
          label="DOWNLOAD"
          accent="blue"
          value={`${formatBytes(snapshot().downloadSpeed)}/s`}
          detail="Live RPC throughput"
        >
          <LiveChart compact values={aria2.downloadHistory()} />
        </StatCard>
      </View>

      <View class="h-40 flex flex-none gap-4">
        <Card class="w-80 flex-none rounded-xl shadow-lg">
          <CardContent class="h-full p-4 flex flex-col gap-3">
            <View class="flex flex-row items-center justify-between">
              <Text class="text-xs font-semibold text-muted">TASKS</Text>
              <Text class="text-xs text-muted">Current session</Text>
            </View>
            <View class="flex-1 flex flex-row items-end gap-3">
              <View class="flex-1 flex flex-col gap-2">
                <Text class="text-3xl font-bold">{activeTasks()}</Text>
                <Text class="text-xs text-muted">Active now</Text>
              </View>
              <View class="flex-1 flex flex-col gap-2">
                <View class="flex flex-row items-center justify-between">
                  <Text class="text-xs text-muted">Waiting</Text>
                  <Text class="text-sm font-semibold">{waitingTasks()}</Text>
                </View>
                <View class="flex flex-row items-center justify-between">
                  <Text class="text-xs text-muted">Completed</Text>
                  <Text class="text-sm font-semibold">{stoppedTasks()}</Text>
                </View>
                <View class="h-2 overflow-hidden rounded-full bg-control">
                  <View
                    class="h-full rounded-full bg-chart-download"
                    style={{
                      width: `${Math.max(8, Math.min(100, activeTasks() * 16))}%`,
                    }}
                  />
                </View>
              </View>
            </View>
          </CardContent>
        </Card>

        <Card class="min-w-0 flex-1 rounded-xl shadow-xl">
          <CardContent class="h-full p-4 flex flex-col gap-2">
            <View class="flex flex-row items-start justify-between">
              <View class="flex flex-col gap-1">
                <Text class="text-xs font-semibold text-muted">
                  TRANSFER OVERVIEW
                </Text>
                <Text class="text-xl font-semibold">
                  {formatBytes(snapshot().downloadedToday)} today
                </Text>
              </View>
              <View class="flex flex-row gap-4">
                <View class="flex flex-row items-center gap-2">
                  <View class="w-2 h-2 rounded-full bg-chart-download" />
                  <Text class="text-xs text-muted">Download</Text>
                </View>
                <View class="flex flex-row items-center gap-2">
                  <View class="w-2 h-2 rounded-full bg-chart-upload" />
                  <Text class="text-xs text-muted">Upload</Text>
                </View>
              </View>
            </View>
            <View class="relative min-h-0 flex-1">
              <LiveChart width={700} values={aria2.downloadHistory()} />
              <View class="absolute inset-0">
                <LiveChart
                  width={700}
                  color="upload"
                  values={aria2.uploadHistory()}
                />
              </View>
            </View>
          </CardContent>
        </Card>
      </View>

      <Card class="min-h-0 flex-1 rounded-xl shadow-xl">
        <CardContent class="h-full p-4 flex flex-col gap-3">
          <View class="flex-none flex flex-row items-start justify-between">
            <View class="flex flex-col gap-1">
              <Text class="text-xs font-semibold text-muted">ACTIVITY</Text>
              <Text class="text-lg font-semibold">A year at a glance</Text>
            </View>
            <View class="flex flex-row items-center gap-3">
              <Text class="text-xs text-muted">
                {activeDays()} active days ·{" "}
                {formatBytes(snapshot().downloadedToday)} today
              </Text>
              <View class="flex flex-row items-center gap-1">
                <Text class="text-xs text-muted">Less</Text>
                <View class="w-2.5 h-2.5 rounded-sm bg-control" />
                <View class="w-2.5 h-2.5 rounded-sm bg-activity-1" />
                <View class="w-2.5 h-2.5 rounded-sm bg-activity-2" />
                <View class="w-2.5 h-2.5 rounded-sm bg-activity-3" />
                <View class="w-2.5 h-2.5 rounded-sm bg-activity-4" />
                <Text class="text-xs text-muted">More</Text>
              </View>
            </View>
          </View>
          <View class="min-h-0 flex-1 flex flex-col items-center justify-center gap-3 rounded-lg bg-surface-muted">
            <View class="w-full px-6 flex flex-row justify-between">
              <Text class="text-xs text-muted">52 weeks ago</Text>
              <Text class="text-xs text-muted">Today</Text>
            </View>
            <View class="flex flex-row items-center justify-center gap-2">
              <View class="flex flex-col gap-2 pr-1">
                <Text class="text-xs text-muted">Mon</Text>
                <Text class="text-xs text-muted">Wed</Text>
                <Text class="text-xs text-muted">Fri</Text>
              </View>
              <View class="flex flex-row gap-1">
                <For each={activityWeeks()}>
                  {(week) => (
                    <View class="flex flex-col gap-1">
                      <For each={week}>
                        {(value) => (
                          <View
                            class="w-2.5 h-2.5 rounded-sm"
                            classList={activityClass(value, activityMaximum())}
                          />
                        )}
                      </For>
                    </View>
                  )}
                </For>
              </View>
            </View>
          </View>
        </CardContent>
      </Card>
    </View>
  );
}
