import {
  application,
  Badge,
  Button,
  Card,
  CardContent,
  createAsyncAction,
  createFormDraft,
  createWindowMatch,
  DirectoryPicker,
  Icon,
  Input,
  PageHeader,
  PrimitiveButton,
  ResponsiveGrid,
  ResponsiveGridRemainder,
  Switch,
  Tabs,
  TabsList,
  TabsTrigger,
  Text,
  useHost,
  useResponsiveGrid,
  View,
} from "@wabou/ui";
import boxes from "lucide-static/icons/boxes.svg?raw";
import download from "lucide-static/icons/download.svg?raw";
import gauge from "lucide-static/icons/gauge.svg?raw";
import info from "lucide-static/icons/info.svg?raw";
import magnet from "lucide-static/icons/magnet.svg?raw";
import radio from "lucide-static/icons/radio-tower.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import sliders from "lucide-static/icons/sliders-horizontal.svg?raw";
import { createEffect, createSignal, For, Show } from "solid-js";
import type { MotrixConfig, MotrixSpeedProfile } from "../downloads";
import { useDownloads } from "../downloads";

type SettingsSection =
  | "general"
  | "appearance"
  | "downloads"
  | "bittorrent"
  | "integration"
  | "network"
  | "advanced"
  | "about";

const settingsItems: readonly [SettingsSection, string, string, string][] = [
  ["general", "General", "Save folder and notifications", settings],
  ["appearance", "Appearance", "Theme and display preferences", gauge],
  ["downloads", "Downloads", "Concurrency and bandwidth", download],
  ["bittorrent", "BitTorrent", "Trackers and peer discovery", magnet],
  ["integration", "Integration", "CLI and desktop behavior", boxes],
  ["network", "Network", "Embedded engine and connectivity", radio],
  ["advanced", "Advanced", "History and maintenance", sliders],
  ["about", "About", "Versions and source code", info],
];

function initialSettingsDraft(config: MotrixConfig) {
  return {
    proxyEnabled: config.proxy.enabled,
    proxyHost: config.proxy.host,
    proxyPort: String(config.proxy.port),
    natEnabled: config.natEnabled,
    natProtocol: config.natProtocol,
    downloadDir: config.downloadDir,
    split: String(config.split),
    connectionsPerServer: String(config.maxConnectionPerServer),
    minSplitSize: config.minSplitSize,
    concurrent: String(config.maxConcurrentDownloads),
    downloadLimit: config.maxOverallDownloadLimit,
    uploadLimit: config.maxOverallUploadLimit,
    speedProfiles: config.speedProfiles.map((profile) => ({ ...profile })),
    userAgent: config.userAgent,
    dhtEnabled: config.dhtEnabled,
    pexEnabled: config.pexEnabled,
    btMaxPeers: String(config.btMaxPeers),
    listenPort: String(config.listenPort),
    seedRatio: String(config.seedRatio),
    notifyComplete: config.notifyOnComplete,
    notifyError: config.notifyOnError,
    resumeOnLaunch: config.resumeAllWhenAppLaunched,
    showDownloadsAfterAdding: config.newTaskShowDownloading,
    warnBeforeQuit: config.warnBeforeQuit,
    theme: config.theme,
  };
}

type SettingsDraft = ReturnType<typeof initialSettingsDraft>;

function validateSettingsDraft(value: Readonly<SettingsDraft>) {
  const errors: Partial<Record<keyof SettingsDraft, string>> = {};
  const integer = (
    key: keyof SettingsDraft,
    label: string,
    input: string,
    minimum: number,
    maximum: number,
  ) => {
    const parsed = Number(input);
    if (!/^\d+$/.test(input.trim()) || !Number.isSafeInteger(parsed))
      errors[key] = `${label} must be a whole number.`;
    else if (parsed < minimum || parsed > maximum)
      errors[key] = `${label} must be between ${minimum} and ${maximum}.`;
  };
  const size = (
    key: keyof SettingsDraft,
    label: string,
    input: string,
    minimum = 0,
  ) => {
    const parsed = parseConfigSize(input);
    if (parsed === undefined || parsed < minimum)
      errors[key] =
        minimum > 0
          ? `${label} must be at least 64K and use an integer K, M, or G size.`
          : `${label} must be 0 or an integer size such as 512K or 10M.`;
  };

  integer("split", "Split count", value.split, 1, 64);
  integer("concurrent", "Concurrent downloads", value.concurrent, 1, 128);
  integer(
    "connectionsPerServer",
    "Connections per server",
    value.connectionsPerServer,
    1,
    64,
  );
  integer("btMaxPeers", "Maximum peers", value.btMaxPeers, 1, 10_000);
  integer("listenPort", "BT listen port", value.listenPort, 1, 65_535);
  size("minSplitSize", "Minimum split size", value.minSplitSize, 64 * 1024);
  size("downloadLimit", "Download limit", value.downloadLimit);
  size("uploadLimit", "Upload limit", value.uploadLimit);
  const ratio = Number(value.seedRatio);
  if (!value.seedRatio.trim() || !Number.isFinite(ratio) || ratio < 0)
    errors.seedRatio = "Seed ratio must be zero or a positive number.";
  if (!value.userAgent.trim())
    errors.userAgent = "HTTP User-Agent is required.";
  if (value.proxyEnabled) {
    if (!value.proxyHost.trim()) errors.proxyHost = "Proxy host is required.";
    integer("proxyPort", "Proxy port", value.proxyPort, 1, 65_535);
  }
  const profileNames = new Set<string>();
  for (const profile of value.speedProfiles) {
    const name = profile.name.trim().toLocaleLowerCase();
    if (!name) {
      errors.speedProfiles = "Every speed profile needs a name.";
      break;
    }
    if (profileNames.has(name)) {
      errors.speedProfiles = "Speed profile names must be unique.";
      break;
    }
    profileNames.add(name);
    if (
      parseConfigSize(profile.downloadLimit) === undefined ||
      parseConfigSize(profile.uploadLimit) === undefined
    ) {
      errors.speedProfiles =
        "Speed profile limits must be 0 or sizes such as 512K or 10M.";
      break;
    }
  }
  return errors;
}

function parseConfigSize(value: string): number | undefined {
  const match = /^(\d+)\s*(B|K|KB|KIB|M|MB|MIB|G|GB|GIB)?$/i.exec(value.trim());
  if (!match) return undefined;
  const number = Number(match[1]);
  const unit = (match[2] ?? "B").toUpperCase();
  const multiplier = unit.startsWith("G")
    ? 1024 ** 3
    : unit.startsWith("M")
      ? 1024 ** 2
      : unit.startsWith("K")
        ? 1024
        : 1;
  const result = number * multiplier;
  return Number.isSafeInteger(result) ? result : undefined;
}

export function SettingsPage() {
  const downloads = useDownloads();
  return (
    <Show
      when={downloads.configStatus() === "ready"}
      fallback={
        <View class="min-w-0 flex flex-col gap-4">
          <Text role="heading" class="text-2xl font-bold">
            Settings
          </Text>
          <Card class="rounded-xl shadow-lg">
            <CardContent class="p-6 flex flex-col items-start gap-3">
              <Text class="font-semibold">
                {downloads.configStatus() === "error"
                  ? "Could not load settings"
                  : "Loading settings…"}
              </Text>
              <Show when={downloads.configError()}>
                {(error) => (
                  <Text
                    role="alert"
                    aria-label="Settings load error"
                    class="text-sm text-danger-primary"
                  >
                    {String(error())}
                  </Text>
                )}
              </Show>
              <Show when={downloads.configStatus() === "error"}>
                <Button
                  onClick={() => void downloads.retryEngine().catch(() => {})}
                >
                  Retry
                </Button>
              </Show>
            </CardContent>
          </Card>
        </View>
      }
    >
      <SettingsForm />
    </Show>
  );
}

function SettingsForm() {
  const downloads = useDownloads();
  const host = useHost();
  const compact = createWindowMatch({ maxWidth: 1100 });
  const snapshot = downloads.snapshot;
  const initialConfig = downloads.config();
  const [section, setSection] = createSignal<SettingsSection>();
  const draft = createFormDraft(initialSettingsDraft(initialConfig), {
    validate: validateSettingsDraft,
  });
  const [proxyEnabled, setProxyEnabled] = draft.control("proxyEnabled");
  const [proxyHost, setProxyHost] = draft.control("proxyHost");
  const [proxyPort, setProxyPort] = draft.control("proxyPort");
  const [natEnabled, setNatEnabled] = draft.control("natEnabled");
  const [natProtocol, setNatProtocol] = draft.control("natProtocol");
  const [downloadDir, setDownloadDir] = draft.control("downloadDir");
  const [split, setSplit] = draft.control("split");
  const [connectionsPerServer, setConnectionsPerServer] = draft.control(
    "connectionsPerServer",
  );
  const [minSplitSize, setMinSplitSize] = draft.control("minSplitSize");
  const [concurrent, setConcurrent] = draft.control("concurrent");
  const [downloadLimit, setDownloadLimit] = draft.control("downloadLimit");
  const [uploadLimit, setUploadLimit] = draft.control("uploadLimit");
  const [speedProfiles, setSpeedProfiles] = draft.control("speedProfiles");
  const [userAgent, setUserAgent] = draft.control("userAgent");
  const [dhtEnabled, setDhtEnabled] = draft.control("dhtEnabled");
  const [pexEnabled, setPexEnabled] = draft.control("pexEnabled");
  const [btMaxPeers, setBtMaxPeers] = draft.control("btMaxPeers");
  const [listenPort, setListenPort] = draft.control("listenPort");
  const [seedRatio, setSeedRatio] = draft.control("seedRatio");
  const [notifyComplete, setNotifyComplete] = draft.control("notifyComplete");
  const [notifyError, setNotifyError] = draft.control("notifyError");
  const [resumeOnLaunch, setResumeOnLaunch] = draft.control("resumeOnLaunch");
  const [showDownloadsAfterAdding, setShowDownloadsAfterAdding] = draft.control(
    "showDownloadsAfterAdding",
  );
  const [warnBeforeQuit, setWarnBeforeQuit] = draft.control("warnBeforeQuit");
  const [theme, setTheme] = draft.control("theme");
  const [message, setMessage] = createSignal("");
  const [restartRequired, setRestartRequired] = createSignal(false);
  const settingsAction = createAsyncAction((operation: () => Promise<void>) =>
    operation(),
  );
  const busy = settingsAction.pending;
  createEffect(downloads.config, (config) => {
    if (!draft.dirty()) draft.resetTo(initialSettingsDraft(config));
  });
  createEffect(draft.dirty, (dirty) => {
    if (dirty) setMessage("");
  });

  const updateSpeedProfile = (
    index: number,
    field: keyof MotrixSpeedProfile,
    value: string,
  ) =>
    setSpeedProfiles((profiles) =>
      profiles.map((profile, position) =>
        position === index ? { ...profile, [field]: value } : profile,
      ),
    );

  const save = async () => {
    if (!draft.valid()) return;
    setMessage("Saving…");
    const outcome = await settingsAction.run(async () => {
      const config: MotrixConfig = {
        ...downloads.config(),
        theme: theme(),
        downloadDir: downloadDir().trim(),
        split: Number.parseInt(split(), 10) || 16,
        maxConnectionPerServer:
          Number.parseInt(connectionsPerServer(), 10) || 16,
        minSplitSize: minSplitSize().trim() || "20M",
        maxConcurrentDownloads: Number.parseInt(concurrent(), 10) || 5,
        notifyOnComplete: notifyComplete(),
        notifyOnError: notifyError(),
        resumeAllWhenAppLaunched: resumeOnLaunch(),
        newTaskShowDownloading: showDownloadsAfterAdding(),
        warnBeforeQuit: warnBeforeQuit(),
        maxOverallDownloadLimit: downloadLimit().trim() || "0",
        maxOverallUploadLimit: uploadLimit().trim() || "0",
        speedProfiles: speedProfiles(),
        userAgent: userAgent().trim() || "Motrix-Wabou/0.1",
        dhtEnabled: dhtEnabled(),
        pexEnabled: pexEnabled(),
        btMaxPeers: Number.parseInt(btMaxPeers(), 10) || 128,
        listenPort: Number.parseInt(listenPort(), 10) || 6881,
        natEnabled: natEnabled(),
        natProtocol: natProtocol(),
        seedRatio: Number.parseFloat(seedRatio()) || 0,
        proxy: {
          enabled: proxyEnabled(),
          host: proxyHost().trim(),
          port: Number.parseInt(proxyPort(), 10) || 8080,
        },
      };
      const result = await downloads.saveConfig(config);
      draft.resetTo(initialSettingsDraft(result.config));
      setMessage(
        result.restartRequired
          ? "Settings saved. Engine changes apply after restart."
          : "Settings saved and applied.",
      );
      setRestartRequired(result.restartRequired);
    });
    if (!outcome.ok) setMessage(String(outcome.error));
  };

  const runMaintenance = async (
    label: string,
    operation: () => Promise<void>,
  ) => {
    setMessage(`${label}…`);
    const outcome = await settingsAction.run(async () => {
      await operation();
      setMessage(`${label} completed.`);
    });
    if (!outcome.ok) setMessage(String(outcome.error));
  };

  return (
    <View class="h-full min-w-0 flex flex-col gap-4">
      <PageHeader
        title="Settings"
        actions={
          <Show when={section()}>
            <Button
              variant="ghost"
              aria-label="Back to settings categories"
              onClick={() => setSection(undefined)}
            >
              All settings
            </Button>
          </Show>
        }
      />

      <Show
        when={section()}
        fallback={
          <SettingsOverview compact={compact()} onSelect={setSection} />
        }
      >
        <View class="min-w-0 flex flex-col gap-4">
          <Tabs
            class="gap-0"
            value={section() ?? "general"}
            onValueChange={(value) => setSection(value as SettingsSection)}
          >
            <TabsList
              unstyled
              aria-label="Settings sections"
              class="grid grid-cols-4 gap-2"
            >
              <For each={settingsItems}>
                {([id, name, detail, icon]) => (
                  <TabsTrigger
                    unstyled
                    value={id}
                    aria-label={`Configure ${name}`}
                    class={(state) =>
                      `w-full min-w-0 p-0 justify-start overflow-hidden rounded-xl border bg-surface shadow-sm ${compact() ? "h-14" : "h-20"} ${section() === id ? "border-accent" : "border-subtle"} ${state.focusVisible ? "border-focus" : ""}`
                    }
                  >
                    <View class="w-full h-full p-3 flex flex-row items-center gap-2">
                      <View
                        class={`w-8 h-8 flex-none rounded-lg flex items-center justify-center ${section() === id ? "bg-accent text-on-accent" : "bg-control text-accent"}`}
                      >
                        <Icon source={icon} size={17} />
                      </View>
                      <View class="min-w-0 flex flex-col items-start gap-1">
                        <Text class="font-semibold">{name}</Text>
                        <Show when={!compact()}>
                          <Text class="truncate text-xs text-muted">
                            {detail}
                          </Text>
                        </Show>
                      </View>
                    </View>
                  </TabsTrigger>
                )}
              </For>
            </TabsList>
          </Tabs>

          <Card class="rounded-2xl shadow-md">
            <CardContent class="p-4 flex flex-col gap-4">
              <Show when={section() === "general"}>
                <SectionHeading
                  title="General"
                  detail="Choose where downloads are stored and which events should notify you."
                />
                <FieldLabel label="Default download directory">
                  <DirectoryPicker
                    aria-label="Default download directory"
                    value={downloadDir()}
                    placeholder="Use downloads default"
                    browseAriaLabel="Browse default download directory"
                    dialogOptions={{
                      title: "Choose the default download directory",
                    }}
                    onValueChange={setDownloadDir}
                    onBrowseError={(error) => setMessage(String(error))}
                  />
                </FieldLabel>
                <View
                  class="flex"
                  classList={{
                    "flex-col gap-3": compact(),
                    "gap-8": !compact(),
                  }}
                >
                  <Switch
                    label="Notify when downloads finish"
                    checked={notifyComplete()}
                    onCheckedChange={setNotifyComplete}
                  />
                  <Switch
                    label="Notify when downloads fail"
                    checked={notifyError()}
                    onCheckedChange={setNotifyError}
                  />
                </View>
                <View
                  class="flex"
                  classList={{
                    "flex-col gap-3": compact(),
                    "gap-8": !compact(),
                  }}
                >
                  <Switch
                    label="Resume paused tasks when Motrix starts"
                    checked={resumeOnLaunch()}
                    onCheckedChange={setResumeOnLaunch}
                  />
                  <Switch
                    label="Show downloads after creating a task"
                    checked={showDownloadsAfterAdding()}
                    onCheckedChange={setShowDownloadsAfterAdding}
                  />
                </View>
                <Switch
                  label="Warn before quitting while downloads are running"
                  checked={warnBeforeQuit()}
                  onCheckedChange={setWarnBeforeQuit}
                />
              </Show>

              <Show when={section() === "appearance"}>
                <SectionHeading
                  title="Appearance"
                  detail="Select the application color theme. The choice is saved for the next launch."
                />
                <ResponsiveGrid
                  role="group"
                  aria-label="Theme choices"
                  minColumnWidth={220}
                  gap={16}
                  maxColumns={3}
                  initialColumns={compact() ? 2 : 3}
                >
                  <ThemeChoice
                    name="System"
                    detail="Follow the native window preference"
                    selected={theme() === "system"}
                    onSelect={() => setTheme("system")}
                  />
                  <ThemeChoice
                    name="Light"
                    detail="Bright surfaces with high-contrast text"
                    selected={theme() === "light"}
                    onSelect={() => setTheme("light")}
                  />
                  <ThemeChoice
                    name="Dark"
                    detail="Low-glare surfaces for dim environments"
                    selected={theme() === "dark"}
                    onSelect={() => setTheme("dark")}
                  />
                </ResponsiveGrid>
              </Show>

              <Show when={section() === "downloads"}>
                <SectionHeading
                  title="Downloads"
                  detail="Tune connection splitting, queue concurrency, and global bandwidth limits."
                />
                <View class="w-full min-w-0 grid grid-cols-2 gap-4">
                  <FieldLabel label="Split count">
                    <Input
                      aria-label="Default split count"
                      value={split()}
                      placeholder="16"
                      onInput={(event) => setSplit(event.currentTarget.value)}
                    />
                  </FieldLabel>
                  <FieldLabel label="Concurrent downloads">
                    <Input
                      aria-label="Concurrent downloads"
                      value={concurrent()}
                      placeholder="5"
                      onInput={(event) =>
                        setConcurrent(event.currentTarget.value)
                      }
                    />
                  </FieldLabel>
                  <FieldLabel label="Connections per server">
                    <Input
                      aria-label="Connections per server"
                      value={connectionsPerServer()}
                      placeholder="16"
                      onInput={(event) =>
                        setConnectionsPerServer(event.currentTarget.value)
                      }
                    />
                  </FieldLabel>
                  <FieldLabel label="Minimum split size">
                    <Input
                      aria-label="Minimum split size"
                      value={minSplitSize()}
                      placeholder="20M"
                      onInput={(event) =>
                        setMinSplitSize(event.currentTarget.value)
                      }
                    />
                  </FieldLabel>
                  <FieldLabel label="Download limit">
                    <Input
                      aria-label="Maximum download speed"
                      value={downloadLimit()}
                      placeholder="0 or 10M"
                      onInput={(event) =>
                        setDownloadLimit(event.currentTarget.value)
                      }
                    />
                  </FieldLabel>
                  <FieldLabel label="Upload limit">
                    <Input
                      aria-label="Maximum upload speed"
                      value={uploadLimit()}
                      placeholder="0 or 1M"
                      onInput={(event) =>
                        setUploadLimit(event.currentTarget.value)
                      }
                    />
                  </FieldLabel>
                  <FieldLabel label="HTTP User-Agent">
                    <Input
                      aria-label="HTTP User-Agent"
                      value={userAgent()}
                      placeholder="Motrix-Wabou/0.1"
                      onInput={(event) =>
                        setUserAgent(event.currentTarget.value)
                      }
                    />
                  </FieldLabel>
                </View>
                <View class="w-full flex flex-col gap-3 rounded-xl border border-subtle bg-surface p-4">
                  <View class="flex items-center justify-between gap-3">
                    <View class="flex flex-col gap-1">
                      <Text class="font-semibold">Speed profiles</Text>
                      <Text class="text-xs text-muted">
                        Save paired download and upload limits for one-click use
                        on the Dashboard.
                      </Text>
                    </View>
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={speedProfiles().length >= 8}
                      onClick={() =>
                        setSpeedProfiles((profiles) => [
                          ...profiles,
                          {
                            name: `Profile ${profiles.length + 1}`,
                            downloadLimit: "0",
                            uploadLimit: "0",
                          },
                        ])
                      }
                    >
                      Add profile
                    </Button>
                  </View>
                  <View class="grid grid-cols-4 gap-2">
                    <Text class="text-xs font-semibold text-muted">Name</Text>
                    <Text class="text-xs font-semibold text-muted">
                      Download
                    </Text>
                    <Text class="text-xs font-semibold text-muted">Upload</Text>
                    <Text class="text-xs font-semibold text-muted">Action</Text>
                    <For each={speedProfiles()}>
                      {(profile, index) => (
                        <>
                          <Input
                            aria-label={`Speed profile ${index() + 1} name`}
                            value={profile.name}
                            onInput={(event) =>
                              updateSpeedProfile(
                                index(),
                                "name",
                                event.currentTarget.value,
                              )
                            }
                          />
                          <Input
                            aria-label={`${profile.name || `Profile ${index() + 1}`} download limit`}
                            value={profile.downloadLimit}
                            placeholder="0 or 10M"
                            onInput={(event) =>
                              updateSpeedProfile(
                                index(),
                                "downloadLimit",
                                event.currentTarget.value,
                              )
                            }
                          />
                          <Input
                            aria-label={`${profile.name || `Profile ${index() + 1}`} upload limit`}
                            value={profile.uploadLimit}
                            placeholder="0 or 1M"
                            onInput={(event) =>
                              updateSpeedProfile(
                                index(),
                                "uploadLimit",
                                event.currentTarget.value,
                              )
                            }
                          />
                          <Button
                            size="sm"
                            variant="ghost"
                            disabled={speedProfiles().length === 1}
                            aria-label={`Remove ${profile.name || `profile ${index() + 1}`}`}
                            onClick={() =>
                              setSpeedProfiles((profiles) =>
                                profiles.filter(
                                  (_profile, position) => position !== index(),
                                ),
                              )
                            }
                          >
                            Remove
                          </Button>
                        </>
                      )}
                    </For>
                  </View>
                </View>
                <Text class="text-xs text-muted">
                  downloads accepts values such as 512K and 10M. Use 0 for
                  unlimited.
                </Text>
              </Show>

              <Show when={section() === "bittorrent"}>
                <SectionHeading
                  title="BitTorrent"
                  detail="Peer discovery and seeding options supported by the embedded engine."
                />
                <View class="w-full min-w-0 grid grid-cols-2 gap-4">
                  <Switch
                    label="Enable DHT peer discovery"
                    checked={dhtEnabled()}
                    onCheckedChange={setDhtEnabled}
                  />
                  <Switch
                    label="Enable peer exchange (PEX)"
                    checked={pexEnabled()}
                    onCheckedChange={setPexEnabled}
                  />
                  <FieldLabel label="Maximum peers per torrent">
                    <Input
                      aria-label="Maximum peers per torrent"
                      value={btMaxPeers()}
                      placeholder="128"
                      onInput={(event) =>
                        setBtMaxPeers(event.currentTarget.value)
                      }
                    />
                  </FieldLabel>
                  <FieldLabel label="BT listen port">
                    <Input
                      aria-label="BT listen port"
                      value={listenPort()}
                      placeholder="6881"
                      onInput={(event) =>
                        setListenPort(event.currentTarget.value)
                      }
                    />
                  </FieldLabel>
                  <FieldLabel label="Seed ratio">
                    <Input
                      aria-label="Seed ratio"
                      value={seedRatio()}
                      placeholder="1.0"
                      onInput={(event) =>
                        setSeedRatio(event.currentTarget.value)
                      }
                    />
                  </FieldLabel>
                </View>
              </Show>

              <Show when={section() === "integration"}>
                <SectionHeading
                  title="Integration"
                  detail="The app embeds gosh-dl and shuts it down cleanly on exit."
                />
                <View class="w-full min-w-0 grid grid-cols-2 gap-4">
                  <InfoCard
                    title="Command line"
                    detail="Launch with Wabou during development or run the packaged application directly."
                  />
                  <InfoCard
                    title="Desktop services"
                    detail="Native file dialogs, dropped torrent files, clipboard access, and notifications are enabled."
                  />
                </View>
              </Show>

              <Show when={section() === "network"}>
                <SectionHeading
                  title="Network and engine"
                  detail="Configure the embedded Rust download engine and incoming peer connectivity."
                />
                <View class="p-4 flex items-center justify-between rounded-lg border border-subtle bg-surface-muted">
                  <View class="min-w-0 flex flex-col gap-1">
                    <Text class="font-semibold">Download service</Text>
                    <Text class="truncate text-sm text-muted">
                      Embedded locally · {snapshot().version ?? "gosh-dl"}
                    </Text>
                  </View>
                  <Badge
                    variant={
                      snapshot().status === "ready" ? "success" : "secondary"
                    }
                  >
                    {snapshot().status === "ready"
                      ? "Ready"
                      : snapshot().status === "starting"
                        ? "Starting"
                        : "Unavailable"}
                  </Badge>
                </View>
                <View class="border-t border-subtle" />
                <View class="flex items-center justify-between">
                  <View class="flex flex-col gap-1">
                    <Text class="font-semibold">Incoming peer connections</Text>
                    <Text class="text-xs text-muted">
                      Maintain TCP and UDP mappings for the configured BT and
                      DHT ports.
                    </Text>
                  </View>
                  <Switch
                    label="Enable automatic port mapping"
                    checked={natEnabled()}
                    onCheckedChange={setNatEnabled}
                  />
                </View>
                <Show when={natEnabled()}>
                  <View class="grid grid-cols-4 gap-2">
                    <For
                      each={
                        [
                          ["auto", "Automatic"],
                          ["pcp", "PCP"],
                          ["natPmp", "NAT-PMP"],
                          ["upnp", "UPnP"],
                        ] as const
                      }
                    >
                      {([value, label]) => (
                        <Button
                          size="sm"
                          variant={
                            natProtocol() === value ? "default" : "outline"
                          }
                          onClick={() => setNatProtocol(value)}
                        >
                          {label}
                        </Button>
                      )}
                    </For>
                  </View>
                  <View
                    role="status"
                    aria-label="Port mapping status"
                    class="p-3 flex flex-col gap-1 rounded-lg bg-surface-muted"
                  >
                    <View class="flex items-center justify-between">
                      <Text class="text-sm font-medium">Port mapping</Text>
                      <Badge
                        variant={
                          snapshot().nat.state === "mapped"
                            ? "success"
                            : "secondary"
                        }
                      >
                        {snapshot().nat.state}
                      </Badge>
                    </View>
                    <Text class="text-xs text-muted">
                      TCP {snapshot().nat.tcpExternalAddress ?? "discovering"} ·
                      UDP {snapshot().nat.udpExternalAddress ?? "discovering"}
                    </Text>
                  </View>
                </Show>
                <View class="border-t border-subtle" />
                <View class="flex items-center justify-between">
                  <View class="flex flex-col gap-1">
                    <Text class="font-semibold">Download proxy</Text>
                    <Text class="text-xs text-muted">
                      Route downloads through an explicit proxy.
                    </Text>
                  </View>
                  <Switch
                    label="Enable download proxy"
                    checked={proxyEnabled()}
                    onCheckedChange={setProxyEnabled}
                  />
                </View>
                <Show when={proxyEnabled()}>
                  <Text class="text-xs text-muted">
                    The engine uses an HTTP forward proxy for HTTP, HTTPS, and
                    FTP downloads.
                  </Text>
                  <View class="w-full min-w-0 grid grid-cols-2 gap-3">
                    <FieldLabel label="Proxy host">
                      <Input
                        aria-label="Proxy host"
                        value={proxyHost()}
                        placeholder="127.0.0.1"
                        onInput={(event) =>
                          setProxyHost(event.currentTarget.value)
                        }
                      />
                    </FieldLabel>
                    <FieldLabel label="Proxy port">
                      <Input
                        aria-label="Proxy port"
                        value={proxyPort()}
                        placeholder="8080"
                        onInput={(event) =>
                          setProxyPort(event.currentTarget.value)
                        }
                      />
                    </FieldLabel>
                  </View>
                </Show>
              </Show>

              <Show when={section() === "advanced"}>
                <SectionHeading
                  title="Advanced"
                  detail="Inspect local application data and configuration."
                />
                <View class="w-full min-w-0 grid grid-cols-1 gap-4">
                  <MaintenanceCard
                    title="Configuration folder"
                    detail="Open the platform-native directory containing Motrix's protected configuration file."
                    action="Open folder"
                    disabled={busy()}
                    onAction={() =>
                      runMaintenance("Open configuration folder", () =>
                        downloads.openConfigFolder(),
                      )
                    }
                  />
                </View>
              </Show>

              <Show when={section() === "about"}>
                <SectionHeading
                  title="About Motrix · Wabou"
                  detail="A native download manager built as a real-world Wabou application."
                />
                <View class="w-full min-w-0 grid grid-cols-2 gap-4">
                  <InfoCard title="Application" detail="Motrix · Wabou 0.1.0" />
                  <InfoCard
                    title="Download engine"
                    detail={snapshot().version ?? "not connected"}
                  />
                  <InfoCard
                    title="Architecture"
                    detail="Rust owns downloads, files, persistence and native services; Solid owns routed UI state."
                  />
                  <View class="min-w-0 p-4 flex flex-col gap-3 rounded-lg border border-subtle bg-surface-muted">
                    <Text class="font-semibold">Source code</Text>
                    <Text class="whitespace-normal text-sm text-muted">
                      Wabou is developed in the open under the Apache-2.0
                      license.
                    </Text>
                    <Button
                      variant="outline"
                      onClick={() =>
                        host.system.openUrl("https://github.com/SunDoge/wabou")
                      }
                    >
                      Open repository
                    </Button>
                  </View>
                </View>
              </Show>

              <View class="pt-4 flex items-center justify-between border-t border-subtle">
                <Show
                  when={
                    !draft.valid()
                      ? Object.values(draft.errors())[0]
                      : message()
                  }
                  fallback={
                    <Text
                      role="status"
                      aria-label={
                        draft.dirty()
                          ? "Unsaved settings"
                          : "Settings up to date"
                      }
                      class="text-xs text-muted"
                    >
                      {draft.dirty()
                        ? "Unsaved changes."
                        : "Settings are up to date."}
                    </Text>
                  }
                >
                  <Text
                    role={draft.valid() ? "status" : "alert"}
                    aria-label={
                      draft.valid() ? message() : "Settings validation error"
                    }
                    class={
                      draft.valid()
                        ? "text-sm text-muted"
                        : "text-sm text-danger-primary"
                    }
                  >
                    {!draft.valid()
                      ? Object.values(draft.errors())[0]
                      : message()}
                  </Text>
                </Show>
                <View class="flex items-center gap-2">
                  <Show when={restartRequired() && !import.meta.env.DEV}>
                    <Button
                      variant="outline"
                      onClick={() => application.relaunch()}
                    >
                      Restart now
                    </Button>
                  </Show>
                  <Button
                    disabled={busy() || !draft.dirty() || !draft.valid()}
                    onClick={save}
                  >
                    {busy() ? "Working…" : "Save settings"}
                  </Button>
                </View>
              </View>
            </CardContent>
          </Card>
        </View>
      </Show>
    </View>
  );
}

function SettingsOverview(props: {
  compact: boolean;
  onSelect(section: SettingsSection): void;
}) {
  return (
    <ResponsiveGrid
      role="group"
      aria-label="Settings categories"
      minColumnWidth={260}
      gap={1}
      maxColumns={3}
      initialColumns={props.compact ? 2 : 3}
      class="flex-none overflow-hidden rounded-2xl bg-subtle shadow-sm"
    >
      <For each={settingsItems}>
        {([id, name, detail, icon]) => (
          <SettingsOverviewItem
            id={id}
            name={name}
            detail={detail}
            icon={icon}
            onSelect={props.onSelect}
          />
        )}
      </For>
      <ResponsiveGridRemainder
        itemCount={settingsItems.length}
        class="bg-surface"
      />
    </ResponsiveGrid>
  );
}

function SettingsOverviewItem(props: {
  id: SettingsSection;
  name: string;
  detail: string;
  icon: string;
  onSelect(section: SettingsSection): void;
}) {
  const grid = useResponsiveGrid();
  const compact = () => grid.columns() < 3;
  return (
    <PrimitiveButton
      unstyled
      aria-label={`Open ${props.name} settings`}
      class={(state) =>
        `w-full min-w-0 ${compact() ? "p-4" : "p-8"} ${state.hovered ? "bg-control-hover" : "bg-surface"}`
      }
      classList={{ "min-h-24": compact(), "min-h-56": !compact() }}
      onClick={() => props.onSelect(props.id)}
    >
      <View
        class="w-full min-w-0 flex items-start gap-4"
        classList={{
          "flex-row items-center": compact(),
          "flex-col": !compact(),
        }}
      >
        <View
          class="flex-none flex items-center justify-center"
          classList={{
            "w-12 h-12 rounded-xl": compact(),
            "w-14 h-14 rounded-2xl": !compact(),
            "bg-selected": ["general", "integration", "about"].includes(
              props.id,
            ),
            "bg-activity-1": ["appearance", "network"].includes(props.id),
            "bg-danger-surface": props.id === "downloads",
            "bg-success-surface": ["bittorrent", "advanced"].includes(props.id),
            "text-accent": props.id === "general",
            "text-chart-download": ["appearance", "network"].includes(props.id),
            "text-danger-primary": props.id === "downloads",
            "text-success-primary": ["bittorrent", "advanced"].includes(
              props.id,
            ),
            "text-chart-upload": props.id === "integration",
            "text-secondary": props.id === "about",
          }}
        >
          <Icon source={props.icon} size={compact() ? 24 : 28} />
        </View>
        <View class="min-w-0 flex flex-col items-start gap-1">
          <Text
            classList={{ "text-base": compact(), "text-lg": !compact() }}
            class="font-semibold text-primary"
          >
            {props.name}
          </Text>
          <Text class="whitespace-normal text-sm text-muted">
            {props.detail}
          </Text>
        </View>
      </View>
    </PrimitiveButton>
  );
}

function SectionHeading(props: { title: string; detail: string }) {
  return (
    <View class="flex flex-col gap-1">
      <Text class="text-lg font-semibold">{props.title}</Text>
      <Text class="whitespace-normal text-sm text-muted">{props.detail}</Text>
    </View>
  );
}

function FieldLabel(props: { label: string; children: unknown }) {
  return (
    <View class="min-w-0 flex flex-col gap-2">
      <Text class="text-sm font-medium">{props.label}</Text>
      {props.children}
    </View>
  );
}

function ThemeChoice(props: {
  name: string;
  detail: string;
  selected: boolean;
  onSelect(): void;
}) {
  return (
    <Button
      aria-label={`Use ${props.name} theme`}
      variant={props.selected ? "default" : "outline"}
      class="min-w-0 min-h-24 px-5 py-4 flex-col items-start justify-center gap-2"
      onClick={props.onSelect}
    >
      <Text class="font-semibold">{props.name}</Text>
      <Text class="w-full min-w-0 whitespace-normal text-left text-xs">
        {props.detail}
      </Text>
    </Button>
  );
}

function InfoCard(props: { title: string; detail: string }) {
  return (
    <View class="min-w-0 p-4 flex flex-col gap-2 rounded-lg border border-subtle bg-surface-muted">
      <Text class="font-semibold">{props.title}</Text>
      <Text class="whitespace-normal text-sm text-muted">{props.detail}</Text>
    </View>
  );
}

function MaintenanceCard(props: {
  title: string;
  detail: string;
  action: string;
  disabled: boolean;
  onAction(): void;
}) {
  return (
    <View class="min-w-0 p-4 flex flex-col gap-3 rounded-lg border border-subtle bg-surface-muted">
      <Text class="font-semibold">{props.title}</Text>
      <Text class="whitespace-normal text-sm text-muted">{props.detail}</Text>
      <Button
        variant="outline"
        disabled={props.disabled}
        onClick={props.onAction}
      >
        {props.action}
      </Button>
    </View>
  );
}
