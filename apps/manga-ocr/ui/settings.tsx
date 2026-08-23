import {
  Badge,
  Button,
  Card,
  CardContent,
  Input,
  Progress,
  Text,
  View,
} from "@wabou/ui";
import { createMemo, Show } from "solid-js";
import { useMangaSession } from "./session";
import { downloadProgressPercent } from "./workflow-state";

function formatBytes(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KiB`;
  return `${(value / 1024 / 1024).toFixed(1)} MiB`;
}

export function Settings() {
  const session = useMangaSession();
  const progressValue = createMemo(() =>
    downloadProgressPercent(session.downloadProgress()),
  );
  const downloading = () => session.busy() === "model";

  return (
    <View class="w-full h-full p-8 items-center bg-canvas">
      <View class="w-full max-w-4xl flex flex-col gap-5">
        <View class="flex flex-col gap-1">
          <Text class="text-3xl font-semibold">Settings</Text>
          <Text class="text-muted">
            Configure native OCR resources and the translation provider.
          </Text>
        </View>
        <Card>
          <CardContent class="p-5 gap-4">
            <View class="flex flex-row items-center gap-2">
              <Text class="font-semibold">PP-OCRv6 small</Text>
              <Badge
                variant={session.modelInstalled() ? "success" : "secondary"}
              >
                {session.modelInstalled() ? "Installed" : "Not installed"}
              </Badge>
              <View class="flex-1" />
              <Text class="text-xs text-muted">{session.modelVersion()}</Text>
            </View>
            <Text class="text-sm text-muted">
              The detector, recognizer, and dictionary are verified before they
              become active.
            </Text>
            <Show
              when={
                downloading() || session.downloadProgress().state === "failed"
              }
            >
              <View class="flex flex-col gap-2">
                <Progress value={progressValue()} maxValue={100} class="h-2" />
                <View class="flex flex-row items-center gap-2">
                  <Text class="text-xs text-muted">
                    {session.downloadProgress().currentFile ??
                      session.downloadProgress().state}
                  </Text>
                  <View class="flex-1" />
                  <Text class="text-xs text-muted">
                    {formatBytes(session.downloadProgress().downloadedBytes)} /{" "}
                    {formatBytes(session.downloadProgress().totalBytes)}
                  </Text>
                </View>
              </View>
            </Show>
            <Button
              disabled={session.modelInstalled() || session.operation.pending()}
              onClick={() => void session.downloadModel()}
            >
              {downloading()
                ? `Downloading ${progressValue()}%`
                : session.modelInstalled()
                  ? "Model installed"
                  : "Download OCR model"}
            </Button>
          </CardContent>
        </Card>
        <Card>
          <CardContent class="p-5 gap-4">
            <Text class="font-semibold">Appearance</Text>
            <Text class="text-sm text-muted">
              Light is the default for image work; dark remains available for
              low-light environments.
            </Text>
            <View class="flex flex-row gap-2">
              <Button
                variant={session.theme() === "light" ? "secondary" : "outline"}
                onClick={() => session.setTheme("light")}
              >
                Light
              </Button>
              <Button
                variant={session.theme() === "dark" ? "secondary" : "outline"}
                onClick={() => session.setTheme("dark")}
              >
                Dark
              </Button>
            </View>
          </CardContent>
        </Card>
        <Card>
          <CardContent class="p-5 gap-4">
            <Text class="font-semibold">LLM translation</Text>
            <Text class="text-sm text-muted">
              Credentials stay in this process for now and are sent only to
              OpenRouter.
            </Text>
            <Input
              aria-label="OpenRouter API key"
              value={session.apiKey()}
              placeholder="OpenRouter API key"
              onInput={(event) => session.setApiKey(event.currentTarget.value)}
            />
            <Input
              aria-label="Translation model"
              value={session.model()}
              onInput={(event) => session.setModel(event.currentTarget.value)}
            />
          </CardContent>
        </Card>
      </View>
    </View>
  );
}
