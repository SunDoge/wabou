import {
  Button,
  dialog,
  DirectoryPicker,
  notification,
  Text,
  View,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import "virtual:wabou-stylesheet";

import { Preview } from "../preview";
import { PropertyRow, ThemeText } from "./showcase";

type SystemAction = "open" | "save" | "directory" | "message" | "notification";

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function SystemPage() {
  const [pending, setPending] = createSignal<SystemAction>();
  const [result, setResult] = createSignal(
    "Choose an action to call the operating-system API.",
  );
  const [failed, setFailed] = createSignal(false);
  const [directory, setDirectory] = createSignal("");

  async function run(
    action: SystemAction,
    operation: () => Promise<string>,
  ): Promise<void> {
    if (pending()) return;
    setPending(action);
    setFailed(false);
    try {
      setResult(await operation());
    } catch (error) {
      setFailed(true);
      setResult(errorMessage(error));
    } finally {
      setPending(undefined);
    }
  }

  const isPending = (action: SystemAction) => pending() === action;
  const disabled = () => pending() !== undefined;

  return (
    <View class="flex flex-col gap-5">
      <Preview title="Native system UI">
        <View class="w-full flex flex-col gap-5">
          <View class="flex flex-wrap items-center justify-center gap-3">
            <Button
              disabled={disabled()}
              onClick={() =>
                void run("open", async () => {
                  const paths = await dialog.open({
                    title: "Open files in Wabou Gallery",
                    multiple: true,
                    filters: [
                      { name: "Images", extensions: ["png", "jpg", "webp"] },
                      { name: "Text", extensions: ["txt", "md", "json"] },
                    ],
                  });
                  return paths?.length
                    ? `Selected ${paths.length}: ${paths.join(", ")}`
                    : "Open files canceled.";
                })
              }
            >
              {isPending("open") ? "Opening..." : "Open files"}
            </Button>
            <Button
              variant="secondary"
              disabled={disabled()}
              onClick={() =>
                void run("save", async () => {
                  const path = await dialog.save({
                    title: "Save from Wabou Gallery",
                    defaultName: "wabou-demo.json",
                    filters: [{ name: "JSON", extensions: ["json"] }],
                  });
                  return path ? `Save destination: ${path}` : "Save canceled.";
                })
              }
            >
              {isPending("save") ? "Opening..." : "Save file"}
            </Button>
            <Button
              variant="outline"
              disabled={disabled()}
              onClick={() =>
                void run("message", async () => {
                  const answer = await dialog.message({
                    title: "Native message dialog",
                    message: "Did this dialog look native on your desktop?",
                    level: "info",
                    buttons: "yesNoCancel",
                  });
                  return `Message dialog result: ${answer}`;
                })
              }
            >
              {isPending("message") ? "Opening..." : "Show message"}
            </Button>
            <Button
              variant="secondary"
              disabled={disabled()}
              onClick={() =>
                void run("notification", async () => {
                  await notification.show({
                    title: "Wabou Gallery",
                    body: "System notifications are connected to the native host.",
                  });
                  return "Notification submitted to the operating system.";
                })
              }
            >
              {isPending("notification") ? "Sending..." : "Send notification"}
            </Button>
          </View>

          <DirectoryPicker
            aria-label="Gallery directory"
            value={directory()}
            placeholder="Choose or enter a directory"
            dialogOptions={{ title: "Choose a directory for Wabou Gallery" }}
            onValueChange={(path) => {
              setDirectory(path);
              setFailed(false);
              setResult(
                path ? `Selected directory: ${path}` : "Directory cleared.",
              );
            }}
            onBrowseError={(error) => {
              setFailed(true);
              setResult(errorMessage(error));
            }}
          />

          <View class="min-h-20 p-4 flex flex-col justify-center gap-1 rounded-lg border border-slate-700">
            <ThemeText
              dark="text-xs font-medium text-slate-400"
              light="text-xs font-medium text-slate-500"
            >
              Last result
            </ThemeText>
            <Text
              class={
                failed() ? "text-sm text-red-400" : "text-sm text-slate-300"
              }
            >
              {result()}
            </Text>
          </View>
        </View>
      </Preview>

      <View class="overflow-hidden rounded-lg border border-slate-800">
        <PropertyRow
          name="dialog.open"
          value="Promise<readonly string[] | null>"
        />
        <PropertyRow name="dialog.save" value="Promise<string | null>" />
        <PropertyRow
          name="dialog.pickDirectory"
          value="Promise<string | null>"
        />
        <PropertyRow
          name="dialog.message"
          value="Promise<ok | cancel | yes | no>"
        />
        <PropertyRow name="notification.show" value="Promise<void>" />
      </View>
    </View>
  );
}
