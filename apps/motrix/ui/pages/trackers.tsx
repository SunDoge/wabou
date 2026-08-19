import {
  Badge,
  Button,
  Card,
  CardContent,
  Text,
  TextArea,
  View,
} from "@wabou/ui";
import { createSignal, Show } from "solid-js";
import { useAria2 } from "../aria2";

const recommendedTrackers = [
  "udp://tracker.opentrackr.org:1337/announce",
  "udp://open.stealth.si:80/announce",
  "udp://tracker.torrent.eu.org:451/announce",
  "udp://tracker.qu.ax:6969/announce",
  "udp://tracker.dler.org:6969/announce",
  "udp://tracker.bitsearch.to:1337/announce",
];
export function TrackersPage() {
  const aria2 = useAria2();
  const [source, setSource] = createSignal(
    aria2.config().btTrackers.join("\n"),
  );
  const [message, setMessage] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const trackers = () =>
    source()
      .split(/\r?\n/)
      .map((value) => value.trim())
      .filter(Boolean);
  const apply = async () => {
    try {
      await aria2.saveConfig({ ...aria2.config(), btTrackers: trackers() });
      setMessage(`Applied ${trackers().length} trackers to aria2.`);
    } catch (error) {
      setMessage(String(error));
    }
  };
  const sync = async (kind: "best" | "all") => {
    setLoading(true);
    setMessage(`Downloading ${kind} tracker list…`);
    try {
      const response = await fetch(
        `https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_${kind}.txt`,
      );
      if (!response.ok)
        throw new Error(`tracker source returned HTTP ${response.status}`);
      const text = await response.text();
      const values = text
        .split(/\r?\n/)
        .map((value) => value.trim())
        .filter(Boolean);
      if (!values.length)
        throw new Error("tracker source returned an empty list");
      setSource(values.join("\n"));
      setMessage(
        `Downloaded ${values.length} trackers. Review them, then apply.`,
      );
    } catch (error) {
      setMessage(String(error));
    } finally {
      setLoading(false);
    }
  };
  return (
    <View class="flex flex-col gap-4">
      <View class="flex items-center justify-between">
        <View class="flex items-center gap-3">
          <Text role="heading" class="text-2xl font-bold">
            Trackers
          </Text>
          <Badge variant="outline">{trackers().length}</Badge>
        </View>
        <View class="flex gap-2">
          <Button
            variant="outline"
            disabled={loading()}
            onClick={() => sync("best")}
          >
            Sync best
          </Button>
          <Button
            variant="outline"
            disabled={loading()}
            onClick={() => sync("all")}
          >
            Sync all
          </Button>
          <Button
            variant="ghost"
            disabled={loading()}
            onClick={() => setSource(recommendedTrackers.join("\n"))}
          >
            Built-in
          </Button>
          <Button onClick={apply}>Apply to engine</Button>
        </View>
      </View>
      <Card>
        <CardContent class="p-4 flex flex-col gap-3">
          <Text class="text-sm text-muted">
            One tracker URL per line. Saving updates aria2's global BitTorrent
            tracker option without restarting the managed engine.
          </Text>
          <TextArea
            class="h-80 font-mono"
            aria-label="BitTorrent trackers"
            value={source()}
            onInput={(event) => setSource(event.currentTarget.value)}
          />
          <Show when={message()}>
            <Text class="text-sm text-muted">{message()}</Text>
          </Show>
        </CardContent>
      </Card>
    </View>
  );
}
