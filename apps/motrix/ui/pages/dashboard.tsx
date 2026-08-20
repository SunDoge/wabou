import {
  Button,
  Card,
  CardContent,
  createAsyncAction,
  createWindowMatch,
  Icon,
  PageHeader,
  ResponsiveGrid,
  Select,
  Text,
  useNavigate,
  View,
} from "@wabou/ui";
import sliders from "lucide-static/icons/sliders-horizontal.svg?raw";
import { createMemo, createSignal, For, Show } from "solid-js";
import { match, P } from "ts-pattern";
import { LiveChart } from "../components/live-chart";
import { StatCard } from "../components/stat-card";
import { useDownloads } from "../downloads";
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
  const compact = createWindowMatch({ maxWidth: 1050 });
  const narrow = createWindowMatch({ maxWidth: 800 });
  const short = createWindowMatch({ maxHeight: 700 });
  const roomy = createWindowMatch({ minWidth: 1250, minHeight: 880 });
  const spaciousActivity = createWindowMatch({ minWidth: 1250 });
  const navigate = useNavigate();
  const downloads = useDownloads();
  const snapshot = downloads.snapshot;
  const serviceReady = () => snapshot().status === "ready";
  const speedLimit = () => downloads.config().maxOverallDownloadLimit;
  const uploadLimit = () => downloads.config().maxOverallUploadLimit;
  const activeSpeedProfile = createMemo(() => {
    const index = downloads
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
  const [speedProfileFeedback, setSpeedProfileFeedback] = createSignal<{
    kind: "success" | "error";
    text: string;
  }>();
  const speedProfileAction = createAsyncAction(async (value: string) => {
    const profile =
      downloads.config().speedProfiles[Number.parseInt(value, 10)];
    if (!profile) throw new Error("The selected speed profile is unavailable.");
    const result = await downloads.saveConfig({
      ...downloads.config(),
      maxOverallDownloadLimit: profile.downloadLimit,
      maxOverallUploadLimit: profile.uploadLimit,
    });
    return { name: profile.name, restartRequired: result.restartRequired };
  });
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
    setSpeedProfileFeedback(undefined);
    const outcome = await speedProfileAction.run(value);
    setSpeedProfileFeedback(
      outcome.ok
        ? {
            kind: "success",
            text: outcome.value.restartRequired
              ? `Speed profile ${outcome.value.name} applied; other engine changes still need restart.`
              : `Speed profile ${outcome.value.name} applied.`,
          }
        : {
            kind: "error",
            text: `Could not apply speed profile: ${String(outcome.error)}`,
          },
    );
  };

  return (
    <View class="min-h-full flex flex-col gap-4">
      <PageHeader
        title="Dashboard"
        actions={
          <Button
            size="icon"
            variant="ghost"
            aria-label="Open settings"
            onClick={() => navigate({ to: "/settings" })}
          >
            <Icon source={sliders} size={20} />
          </Button>
        }
      />

      <ResponsiveGrid
        minColumnWidth={200}
        gap={roomy() ? 20 : 16}
        maxColumns={4}
        initialColumns={compact() ? 2 : 4}
        class="flex-none"
      >
        <StatCard
          dense={short()}
          roomy={roomy()}
          label="DOWNLOAD SERVICE"
          accent={serviceReady() ? "green" : "neutral"}
          value={serviceReady() ? "Ready" : "Unavailable"}
          detail={snapshot().version ?? "Embedded Rust service"}
          description={
            serviceReady()
              ? "Embedded downloader is responding"
              : (snapshot().error ?? "Starting the embedded downloader")
          }
        />
        <StatCard
          dense={short()}
          roomy={roomy()}
          label="SPEED LIMIT"
          accent="neutral"
          value={speedLimit() === "0" ? "Unlimited" : speedLimit()}
          footer={
            <Show
              when={speedProfileFeedback()}
              fallback={
                <Text class="truncate text-xs text-muted">
                  Upload {uploadLimit() === "0" ? "unlimited" : uploadLimit()}
                </Text>
              }
            >
              {(feedback) => (
                <Text
                  role={feedback().kind === "error" ? "alert" : "status"}
                  aria-label={feedback().text}
                  class={
                    feedback().kind === "error"
                      ? "truncate text-xs text-danger-primary"
                      : "truncate text-xs text-success-primary"
                  }
                >
                  {feedback().text}
                </Text>
              )}
            </Show>
          }
        >
          <Select
            aria-label="Speed profile"
            class="w-full"
            disabled={speedProfileAction.pending()}
            placeholder="Custom limits"
            value={activeSpeedProfile()}
            options={downloads.config().speedProfiles.map((profile, index) => ({
              value: String(index),
              label: profile.name,
            }))}
            onValueChange={(value) => void setSpeedProfile(value)}
          />
        </StatCard>
        <StatCard
          dense={short()}
          roomy={roomy()}
          label="UPLOAD"
          accent="purple"
          value={`${formatBytes(snapshot().uploadSpeed)}/s`}
          detail="Live engine throughput"
        >
          <LiveChart
            compact
            color="upload"
            values={downloads.uploadHistory()}
          />
        </StatCard>
        <StatCard
          dense={short()}
          roomy={roomy()}
          label="DOWNLOAD"
          accent="blue"
          value={`${formatBytes(snapshot().downloadSpeed)}/s`}
          detail="Live engine throughput"
        >
          <LiveChart compact values={downloads.downloadHistory()} />
        </StatCard>
      </ResponsiveGrid>

      <View
        class={
          narrow()
            ? "flex flex-none flex-col gap-4"
            : short()
              ? "h-32 flex flex-none gap-4"
              : compact()
                ? "flex flex-none flex-col gap-4"
                : roomy()
                  ? "h-56 flex flex-none gap-5"
                  : "h-40 flex flex-none gap-4"
        }
      >
        <Card
          role="group"
          aria-label="Task statistics"
          class={
            narrow()
              ? "w-full h-32 flex-none rounded-xl shadow-lg"
              : short()
                ? "w-64 flex-none rounded-xl shadow-lg"
                : compact()
                  ? "w-full h-40 flex-none rounded-xl shadow-lg"
                  : "flex-1 rounded-2xl shadow-md"
          }
        >
          <CardContent
            class={
              roomy()
                ? "h-full p-6 flex flex-col gap-3"
                : "h-full p-4 flex flex-col gap-3"
            }
          >
            <View class="flex flex-row items-center justify-between">
              <Text class="text-sm font-medium text-secondary">
                ACTIVE TASKS
              </Text>
              <View class="w-3 h-3 rounded-full bg-strong" />
            </View>
            <Show
              when={!short()}
              fallback={
                <View class="min-h-0 flex-1 flex flex-row items-end gap-3">
                  <View class="flex-1 flex flex-col gap-1">
                    <Text class="text-3xl font-semibold">{activeTasks()}</Text>
                    <Text class="text-xs text-secondary">DOWNLOADING</Text>
                  </View>
                  <View class="flex-1 flex flex-col gap-2">
                    <View class="flex items-center justify-between">
                      <Text class="text-xs text-secondary">Waiting</Text>
                      <Text class="text-sm font-semibold">
                        {waitingTasks()}
                      </Text>
                    </View>
                    <View class="flex items-center justify-between">
                      <Text class="text-xs text-secondary">Completed</Text>
                      <Text class="text-sm font-semibold">
                        {stoppedTasks()}
                      </Text>
                    </View>
                  </View>
                </View>
              }
            >
              <Text
                class={
                  roomy() ? "text-4xl font-semibold" : "text-3xl font-semibold"
                }
              >
                {activeTasks()}
              </Text>
              <View class="flex-1" />
              <View class="flex flex-row items-end">
                <View class="flex-1 flex flex-col gap-1">
                  <Text class="text-xl font-semibold">{activeTasks()}</Text>
                  <Text class="text-xs text-secondary">DOWNLOADING</Text>
                </View>
                <View class="w-px h-12 mx-4 bg-subtle" />
                <View class="flex-1 flex flex-col gap-1">
                  <Text class="text-xl font-semibold">{waitingTasks()}</Text>
                  <Text class="text-xs text-secondary">WAITING</Text>
                </View>
                <View class="w-px h-12 mx-4 bg-subtle" />
                <View class="flex-1 flex flex-col gap-1">
                  <Text class="text-xl font-semibold">{stoppedTasks()}</Text>
                  <Text class="text-xs text-secondary">COMPLETED</Text>
                </View>
              </View>
            </Show>
          </CardContent>
        </Card>

        <Card
          role="group"
          aria-label="Transfer overview"
          class={
            narrow()
              ? "w-full h-40 flex-none rounded-2xl shadow-md"
              : short()
                ? "min-w-0 flex-1 rounded-2xl shadow-md"
                : compact()
                  ? "w-full h-40 flex-none rounded-2xl shadow-md"
                  : "min-w-0 flex-1 rounded-2xl shadow-md"
          }
        >
          <CardContent class="h-full p-4 flex flex-col gap-2">
            <View class="flex flex-row items-start justify-between">
              <View class="flex flex-col gap-1">
                <Text class="text-sm font-medium text-secondary">
                  TRANSFER OVERVIEW
                </Text>
                <Text
                  class={
                    roomy() ? "text-4xl font-semibold" : "text-xl font-semibold"
                  }
                >
                  {formatBytes(snapshot().downloadedToday)} today
                </Text>
                <Text class="text-xs text-secondary">
                  {formatBytes(snapshot().downloadedTotal)} downloaded ·{" "}
                  {formatBytes(snapshot().uploadedTotal)} uploaded all time
                </Text>
              </View>
              <Show when={!short()}>
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
              </Show>
            </View>
            <View class="relative min-h-0 flex-1">
              <LiveChart values={downloads.downloadHistory()} />
              <View class="absolute inset-0">
                <LiveChart
                  color="upload"
                  grid={false}
                  values={downloads.uploadHistory()}
                />
              </View>
            </View>
          </CardContent>
        </Card>
      </View>

      <Show when={!short()}>
        <Card
          role="group"
          aria-label="Download activity"
          class={
            compact()
              ? "h-80 flex-none rounded-2xl shadow-md"
              : "min-h-0 flex-1 rounded-2xl shadow-md"
          }
        >
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
            <View class="min-h-0 flex-1 flex flex-col items-center justify-center gap-3 rounded-lg bg-surface">
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
                              class={
                                spaciousActivity()
                                  ? "w-3 h-3 rounded-sm"
                                  : "w-2.5 h-2.5 rounded-sm"
                              }
                              classList={activityClass(
                                value,
                                activityMaximum(),
                              )}
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
      </Show>
    </View>
  );
}
