import {
  Badge,
  Button,
  Card,
  CardContent,
  dialog,
  Icon,
  Input,
  Switch,
  Text,
  useHost,
  useNavigate,
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
import { createSignal, For, Show } from "solid-js";
import type { MotrixSpeedProfile } from "../aria2";
import { useAria2 } from "../aria2";

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
  ["network", "Network", "RPC endpoint and engine", radio],
  ["advanced", "Advanced", "History and maintenance", sliders],
  ["about", "About", "Versions and source code", info],
];

export function SettingsPage() {
  const aria2 = useAria2();
  const host = useHost();
  const navigate = useNavigate();
  const snapshot = aria2.snapshot;
  const initialConfig = aria2.config();
  const [section, setSection] = createSignal<SettingsSection>("general");
  const [external, setExternal] = createSignal(
    initialConfig.engineMode === "external",
  );
  const [endpoint, setEndpoint] = createSignal(initialConfig.externalEndpoint);
  const [secret, setSecret] = createSignal(initialConfig.externalSecret);
  const [proxyEnabled, setProxyEnabled] = createSignal(
    initialConfig.proxy.enabled,
  );
  const [proxyHost, setProxyHost] = createSignal(initialConfig.proxy.host);
  const [proxyPort, setProxyPort] = createSignal(
    String(initialConfig.proxy.port),
  );
  const [proxyBypass, setProxyBypass] = createSignal(
    initialConfig.proxy.bypass.join(", "),
  );
  const [natEnabled, setNatEnabled] = createSignal(initialConfig.natEnabled);
  const [natProtocol, setNatProtocol] = createSignal(initialConfig.natProtocol);
  const [downloadDir, setDownloadDir] = createSignal(initialConfig.downloadDir);
  const [split, setSplit] = createSignal(String(initialConfig.split));
  const [connectionsPerServer, setConnectionsPerServer] = createSignal(
    String(initialConfig.maxConnectionPerServer),
  );
  const [minSplitSize, setMinSplitSize] = createSignal(
    initialConfig.minSplitSize,
  );
  const [fileAllocation, setFileAllocation] = createSignal(
    initialConfig.fileAllocation,
  );
  const [concurrent, setConcurrent] = createSignal(
    String(initialConfig.maxConcurrentDownloads),
  );
  const [downloadLimit, setDownloadLimit] = createSignal(
    initialConfig.maxOverallDownloadLimit,
  );
  const [uploadLimit, setUploadLimit] = createSignal(
    initialConfig.maxOverallUploadLimit,
  );
  const [speedProfiles, setSpeedProfiles] = createSignal<MotrixSpeedProfile[]>(
    initialConfig.speedProfiles.map((profile) => ({ ...profile })),
  );
  const [userAgent, setUserAgent] = createSignal(initialConfig.userAgent);
  const [dhtEnabled, setDhtEnabled] = createSignal(initialConfig.dhtEnabled);
  const [pexEnabled, setPexEnabled] = createSignal(initialConfig.pexEnabled);
  const [btMaxPeers, setBtMaxPeers] = createSignal(
    String(initialConfig.btMaxPeers),
  );
  const [listenPort, setListenPort] = createSignal(
    String(initialConfig.listenPort),
  );
  const [dhtListenPort, setDhtListenPort] = createSignal(
    String(initialConfig.dhtListenPort),
  );
  const [seedRatio, setSeedRatio] = createSignal(
    String(initialConfig.seedRatio),
  );
  const [seedTime, setSeedTime] = createSignal(String(initialConfig.seedTime));
  const [notifyComplete, setNotifyComplete] = createSignal(
    initialConfig.notifyOnComplete,
  );
  const [notifyError, setNotifyError] = createSignal(
    initialConfig.notifyOnError,
  );
  const [resumeOnLaunch, setResumeOnLaunch] = createSignal(
    initialConfig.resumeAllWhenAppLaunched,
  );
  const [showDownloadsAfterAdding, setShowDownloadsAfterAdding] = createSignal(
    initialConfig.newTaskShowDownloading,
  );
  const [warnBeforeQuit, setWarnBeforeQuit] = createSignal(
    initialConfig.warnBeforeQuit,
  );
  const [theme, setTheme] = createSignal<"light" | "dark" | "system">(
    initialConfig.theme,
  );
  const [message, setMessage] = createSignal("");
  const [busy, setBusy] = createSignal(false);

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
    setBusy(true);
    setMessage("Saving…");
    try {
      await aria2.saveConfig({
        ...aria2.config(),
        theme: theme(),
        engineMode: external() ? "external" : "managed",
        externalEndpoint: endpoint().trim(),
        externalSecret: secret(),
        downloadDir: downloadDir().trim(),
        split: Number.parseInt(split(), 10) || 16,
        maxConnectionPerServer:
          Number.parseInt(connectionsPerServer(), 10) || 16,
        minSplitSize: minSplitSize().trim() || "20M",
        fileAllocation: fileAllocation(),
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
        dhtListenPort: Number.parseInt(dhtListenPort(), 10) || 6881,
        natEnabled: natEnabled(),
        natProtocol: natProtocol(),
        seedRatio: Number.parseFloat(seedRatio()) || 0,
        seedTime: Number.parseInt(seedTime(), 10) || 0,
        proxy: {
          enabled: proxyEnabled(),
          host: proxyHost().trim(),
          port: Number.parseInt(proxyPort(), 10) || 8080,
          bypass: proxyBypass()
            .split(",")
            .map((entry) => entry.trim())
            .filter(Boolean),
        },
      });
      await aria2.refresh();
      setMessage("Settings saved and engine connection updated.");
    } catch (error) {
      setMessage(String(error));
    } finally {
      setBusy(false);
    }
  };

  const runEngineAction = async (action: "start" | "stop" | "restart") => {
    setBusy(true);
    setMessage(`${action[0].toUpperCase()}${action.slice(1)}ing aria2…`);
    try {
      await aria2.engineAction(action);
      await aria2.refresh();
      setMessage(`aria2 ${action} completed.`);
    } catch (error) {
      setMessage(String(error));
    } finally {
      setBusy(false);
    }
  };

  const runMaintenance = async (
    label: string,
    operation: () => Promise<void>,
  ) => {
    setBusy(true);
    setMessage(`${label}…`);
    try {
      await operation();
      setMessage(`${label} completed.`);
    } catch (error) {
      setMessage(String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <View class="min-w-0 flex flex-col gap-4">
      <View class="flex items-center justify-between">
        <View class="flex flex-col gap-1">
          <Text role="heading" class="text-2xl font-bold">
            Settings
          </Text>
          <Text class="text-sm text-muted">
            Configure Motrix and the aria2 engine from one place.
          </Text>
        </View>
        <Badge variant={snapshot().connected ? "success" : "secondary"}>
          {snapshot().connected ? "Engine connected" : "Engine offline"}
        </Badge>
      </View>

      <View class="grid grid-cols-4 gap-2">
        <For each={settingsItems}>
          {([id, name, detail, icon]) => (
            <Button
              aria-label={`Configure ${name}`}
              variant="ghost"
              class={`min-w-0 h-20 p-0 justify-start rounded-xl ${section() === id ? "bg-selected" : ""}`}
              onClick={() => setSection(id)}
            >
              <Card class="w-full min-w-0 h-full">
                <CardContent class="h-full p-3 flex flex-row items-center gap-2">
                  <View
                    class={`w-8 h-8 flex-none rounded-lg flex items-center justify-center ${section() === id ? "bg-accent text-on-accent" : "bg-control text-accent"}`}
                  >
                    <Icon source={icon} size={17} />
                  </View>
                  <View class="min-w-0 flex flex-col items-start gap-1">
                    <Text class="font-semibold">{name}</Text>
                    <Text class="truncate text-xs text-muted">{detail}</Text>
                  </View>
                </CardContent>
              </Card>
            </Button>
          )}
        </For>
      </View>

      <Card class="rounded-xl shadow-lg">
        <CardContent class="p-4 flex flex-col gap-4">
          <Show when={section() === "general"}>
            <SectionHeading
              title="General"
              detail="Choose where downloads are stored and which events should notify you."
            />
            <FieldLabel label="Default download directory">
              <View class="min-w-0 flex gap-2">
                <Input
                  class="min-w-0 flex-1"
                  aria-label="Default download directory"
                  value={downloadDir()}
                  placeholder="Use aria2 default"
                  onInput={(event) => setDownloadDir(event.currentTarget.value)}
                />
                <Button
                  variant="outline"
                  onClick={async () => {
                    const path = await dialog.pickDirectory({
                      title: "Choose the default download directory",
                    });
                    if (path) setDownloadDir(path);
                  }}
                >
                  Browse…
                </Button>
              </View>
            </FieldLabel>
            <View class="flex gap-8">
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
            <View class="flex gap-8">
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
            <View class="w-full min-w-0 grid grid-cols-3 gap-4">
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
            </View>
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
                  onInput={(event) => setConcurrent(event.currentTarget.value)}
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
                  onInput={(event) => setUploadLimit(event.currentTarget.value)}
                />
              </FieldLabel>
              <FieldLabel label="HTTP User-Agent">
                <Input
                  aria-label="HTTP User-Agent"
                  value={userAgent()}
                  placeholder="Motrix-Wabou/0.1"
                  onInput={(event) => setUserAgent(event.currentTarget.value)}
                />
              </FieldLabel>
              <FieldLabel label="File allocation">
                <View class="grid grid-cols-4 gap-2">
                  <For each={["none", "prealloc", "trunc", "falloc"] as const}>
                    {(value) => (
                      <Button
                        aria-label={`Use ${value} file allocation`}
                        variant={
                          fileAllocation() === value ? "default" : "outline"
                        }
                        onClick={() => setFileAllocation(value)}
                      >
                        {value}
                      </Button>
                    )}
                  </For>
                </View>
              </FieldLabel>
            </View>
            <View class="w-full flex flex-col gap-3 rounded-xl border border-subtle bg-surface p-4">
              <View class="flex items-center justify-between gap-3">
                <View class="flex flex-col gap-1">
                  <Text class="font-semibold">Speed profiles</Text>
                  <Text class="text-xs text-muted">
                    Save paired download and upload limits for one-click use on
                    the Dashboard.
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
                <Text class="text-xs font-semibold text-muted">Download</Text>
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
              aria2 accepts values such as 512K and 10M. Use 0 for unlimited.
            </Text>
          </Show>

          <Show when={section() === "bittorrent"}>
            <SectionHeading
              title="BitTorrent"
              detail="Tracker lists are maintained separately so large lists remain easy to review."
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
                  onInput={(event) => setBtMaxPeers(event.currentTarget.value)}
                />
              </FieldLabel>
              <FieldLabel label="BT listen port">
                <Input
                  aria-label="BT listen port"
                  value={listenPort()}
                  placeholder="6881"
                  onInput={(event) => setListenPort(event.currentTarget.value)}
                />
              </FieldLabel>
              <FieldLabel label="DHT listen port">
                <Input
                  aria-label="DHT listen port"
                  value={dhtListenPort()}
                  placeholder="6881"
                  onInput={(event) =>
                    setDhtListenPort(event.currentTarget.value)
                  }
                />
              </FieldLabel>
              <FieldLabel label="Seed ratio">
                <Input
                  aria-label="Seed ratio"
                  value={seedRatio()}
                  placeholder="1.0"
                  onInput={(event) => setSeedRatio(event.currentTarget.value)}
                />
              </FieldLabel>
              <FieldLabel label="Seed time (minutes)">
                <Input
                  aria-label="Seed time in minutes"
                  value={seedTime()}
                  placeholder="60"
                  onInput={(event) => setSeedTime(event.currentTarget.value)}
                />
              </FieldLabel>
            </View>
            <View class="p-4 flex items-center justify-between rounded-lg border border-subtle bg-surface-muted">
              <View class="flex flex-col gap-1">
                <Text class="font-medium">
                  {aria2.config().btTrackers.length} configured trackers
                </Text>
                <Text class="text-sm text-muted">
                  Sync a curated list or provide one tracker URL per line.
                </Text>
              </View>
              <Button
                variant="outline"
                onClick={() => navigate({ to: "/trackers" })}
              >
                Manage trackers
              </Button>
            </View>
          </Show>

          <Show when={section() === "integration"}>
            <SectionHeading
              title="Integration"
              detail="Motrix owns a local aria2 process by default and shuts it down cleanly on exit."
            />
            <View class="w-full min-w-0 grid grid-cols-2 gap-4">
              <InfoCard
                title="Command line"
                detail="Launch with Wabou during development or run the packaged Motrix binary directly."
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
              detail="Control the managed engine or connect Motrix to another aria2 instance."
            />
            <View class="p-4 flex items-center justify-between rounded-lg border border-subtle bg-surface-muted">
              <View class="min-w-0 flex flex-col gap-1">
                <Text class="font-semibold">aria2 engine</Text>
                <Text class="truncate text-sm text-muted">
                  {snapshot().managed
                    ? "Managed locally"
                    : "External controller"}{" "}
                  · {snapshot().endpoint}
                </Text>
              </View>
              <Badge variant={snapshot().connected ? "success" : "secondary"}>
                {snapshot().connected
                  ? "Connected"
                  : snapshot().engineRunning
                    ? "Connecting"
                    : "Stopped"}
              </Badge>
            </View>
            <View class="flex gap-2">
              <Button
                disabled={
                  busy() || !snapshot().managed || snapshot().engineRunning
                }
                onClick={() => runEngineAction("start")}
              >
                Start
              </Button>
              <Button
                variant="outline"
                disabled={
                  busy() || !snapshot().managed || !snapshot().engineRunning
                }
                onClick={() => runEngineAction("restart")}
              >
                Restart
              </Button>
              <Button
                variant="destructive"
                disabled={
                  busy() || !snapshot().managed || !snapshot().engineRunning
                }
                onClick={() => runEngineAction("stop")}
              >
                Stop
              </Button>
            </View>
            <View class="border-t border-subtle" />
            <Switch
              label="Use an external aria2 controller"
              checked={external()}
              onCheckedChange={setExternal}
            />
            <Show when={external()}>
              <View class="w-full min-w-0 grid grid-cols-2 gap-3">
                <FieldLabel label="WebSocket endpoint">
                  <Input
                    aria-label="aria2 RPC endpoint"
                    value={endpoint()}
                    placeholder="ws://127.0.0.1:6800/jsonrpc"
                    onInput={(event) => setEndpoint(event.currentTarget.value)}
                  />
                </FieldLabel>
                <FieldLabel label="RPC secret">
                  <Input
                    aria-label="aria2 RPC secret"
                    value={secret()}
                    placeholder="Optional"
                    onInput={(event) => setSecret(event.currentTarget.value)}
                  />
                </FieldLabel>
              </View>
            </Show>
            <Text class="text-xs text-muted">
              WABOU_ARIA2_URL and WABOU_ARIA2_SECRET override saved values for
              the current launch.
            </Text>
            <View class="border-t border-subtle" />
            <View class="flex items-center justify-between">
              <View class="flex flex-col gap-1">
                <Text class="font-semibold">Incoming peer connections</Text>
                <Text class="text-xs text-muted">
                  Maintain TCP and UDP mappings for the configured BT and DHT
                  ports.
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
                      variant={natProtocol() === value ? "default" : "outline"}
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
                  TCP {snapshot().nat.tcpExternalAddress ?? "discovering"} · UDP{" "}
                  {snapshot().nat.udpExternalAddress ?? "discovering"}
                </Text>
                <Show when={snapshot().nat.dhtExternalAddress}>
                  {(address) => (
                    <Text class="text-xs text-muted">DHT UDP {address()}</Text>
                  )}
                </Show>
              </View>
            </Show>
            <View class="border-t border-subtle" />
            <View class="flex items-center justify-between">
              <View class="flex flex-col gap-1">
                <Text class="font-semibold">Download proxy</Text>
                <Text class="text-xs text-muted">
                  Route aria2 downloads through an explicit proxy.
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
                aria2 uses an HTTP forward proxy for HTTP, HTTPS, and FTP
                downloads.
              </Text>
              <View class="w-full min-w-0 grid grid-cols-2 gap-3">
                <FieldLabel label="Proxy host">
                  <Input
                    aria-label="Proxy host"
                    value={proxyHost()}
                    placeholder="127.0.0.1"
                    onInput={(event) => setProxyHost(event.currentTarget.value)}
                  />
                </FieldLabel>
                <FieldLabel label="Proxy port">
                  <Input
                    aria-label="Proxy port"
                    value={proxyPort()}
                    placeholder="8080"
                    onInput={(event) => setProxyPort(event.currentTarget.value)}
                  />
                </FieldLabel>
              </View>
              <FieldLabel label="Bypass hosts">
                <Input
                  aria-label="Proxy bypass hosts"
                  value={proxyBypass()}
                  placeholder="localhost, 127.0.0.1"
                  onInput={(event) => setProxyBypass(event.currentTarget.value)}
                />
              </FieldLabel>
            </Show>
          </Show>

          <Show when={section() === "advanced"}>
            <SectionHeading
              title="Advanced"
              detail="Inspect local application data and maintain aria2's completed-task history."
            />
            <View class="w-full min-w-0 grid grid-cols-2 gap-4">
              <MaintenanceCard
                title="Download history"
                detail="Remove completed, failed, and removed task records from aria2. Downloaded files are kept."
                action="Clear history"
                disabled={busy()}
                onAction={() =>
                  runMaintenance("Clear download history", async () => {
                    await aria2.globalTaskAction("clearCompleted");
                    await aria2.refresh();
                  })
                }
              />
              <MaintenanceCard
                title="Configuration folder"
                detail="Open the platform-native directory containing Motrix's protected configuration file."
                action="Open folder"
                disabled={busy()}
                onAction={() =>
                  runMaintenance("Open configuration folder", () =>
                    aria2.openConfigFolder(),
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
                detail={`aria2 ${snapshot().version ?? "not connected"}`}
              />
              <InfoCard
                title="Architecture"
                detail="Rust owns aria2, files, persistence and native services; Solid owns routed UI state."
              />
              <View class="min-w-0 p-4 flex flex-col gap-3 rounded-lg border border-subtle bg-surface-muted">
                <Text class="font-semibold">Source code</Text>
                <Text class="whitespace-normal text-sm text-muted">
                  Wabou is developed in the open under the Apache-2.0 license.
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
              when={message()}
              fallback={
                <Text class="text-xs text-muted">
                  Changes take effect after saving.
                </Text>
              }
            >
              <Text class="text-sm text-muted">{message()}</Text>
            </Show>
            <Button disabled={busy()} onClick={save}>
              {busy() ? "Working…" : "Save settings"}
            </Button>
          </View>
        </CardContent>
      </Card>
    </View>
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
      class="h-24 px-5 flex-col items-start justify-center gap-2"
      onClick={props.onSelect}
    >
      <Text class="font-semibold">{props.name}</Text>
      <Text class="text-xs">{props.detail}</Text>
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
