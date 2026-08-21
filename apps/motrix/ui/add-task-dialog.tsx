import {
  Button,
  Checkbox,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  clipboard,
  createFormDraft,
  createLatestAsyncResource,
  DialogScrollBody,
  DirectoryPicker,
  dialog,
  Input,
  Modal,
  Text,
  TextArea,
  ToggleGroup,
  ToggleGroupItem,
  View,
  VirtualList,
} from "@wabou/ui";
import { createEffect, createSignal, Show, untrack } from "solid-js";
import type { MotrixConfig, TaskPriority, TorrentPreview } from "./downloads";
import { useDownloads } from "./downloads";
import { parseCurlDownload } from "./lib/curl";
import { downloadUriError, downloadUris } from "./lib/download-uri";
import { formatBytes } from "./lib/format";

function initialAddTaskDraft(config: MotrixConfig) {
  return {
    source: "links" as "links" | "torrent",
    url: "",
    torrentPath: "",
    selectedTorrentFiles: [] as number[],
    directory: config.downloadDir,
    filename: "",
    split: String(config.split),
    headers: "",
    priority: "normal" as TaskPriority,
  };
}

type AddTaskDraft = ReturnType<typeof initialAddTaskDraft>;

function validateAddTaskDraft(value: Readonly<AddTaskDraft>) {
  const errors: Partial<Record<keyof AddTaskDraft, string>> = {};
  const split = Number(value.split);
  if (!/^\d+$/.test(value.split.trim()) || !Number.isSafeInteger(split))
    errors.split = "Split count must be a whole number.";
  else if (split < 1 || split > 64)
    errors.split = "Split count must be between 1 and 64.";
  if (
    value.source === "links" &&
    value.filename.trim() &&
    downloadUris(value.url).length > 1
  )
    errors.filename = "A custom output filename can only be used with one URL.";
  if (value.source === "links" && value.url.trim()) {
    const error = downloadUriError(value.url);
    if (error) errors.url = error;
  }
  if (
    value.source === "links" &&
    value.headers
      .split(/\r?\n/)
      .map((header) => header.trim())
      .filter(Boolean)
      .some(
        (header) => !header.includes(":") || !header.split(":", 1)[0]?.trim(),
      )
  )
    errors.headers = "Each HTTP header must use the Name: value format.";
  return errors;
}

export interface AddTaskDialogProps {
  open: boolean;
  initialSource: "links" | "torrent";
  initialTorrentPath?: string;
  onOpenChange(open: boolean): void;
  onCreated(): void | Promise<void>;
}

export function AddTaskDialog(props: AddTaskDialogProps) {
  const downloads = useDownloads();
  const addTask = createFormDraft(
    initialAddTaskDraft(untrack(downloads.config)),
    {
      validate: validateAddTaskDraft,
    },
  );
  const [source, setSource] = addTask.control("source");
  const [url, setUrl] = addTask.control("url");
  const [torrentPath, setTorrentPath] = addTask.control("torrentPath");
  const [selectedTorrentFiles, setSelectedTorrentFiles] = addTask.control(
    "selectedTorrentFiles",
  );
  const [directory, setDirectory] = addTask.control("directory");
  const [filename, setFilename] = addTask.control("filename");
  const [split, setSplit] = addTask.control("split");
  const [headers, setHeaders] = addTask.control("headers");
  const [priority, setPriority] = addTask.control("priority");
  const [addError, setAddError] = createSignal("");
  const [addNotice, setAddNotice] = createSignal("");
  const torrentInspection = createLatestAsyncResource<string, TorrentPreview>({
    source: () => torrentPath() || undefined,
    load: (path) => downloads.inspectTorrent(path),
  });
  const torrentPreview = torrentInspection.value;
  const inspectingTorrent = torrentInspection.loading;

  createEffect(torrentInspection.value, (preview) => {
    if (preview)
      setSelectedTorrentFiles(preview.files.map((file) => file.index));
  });
  createEffect(torrentInspection.error, (error) => {
    if (error) setAddError(String(error));
  });
  createEffect(
    () => [props.open, props.initialSource, props.initialTorrentPath] as const,
    ([open, initialSource, initialTorrentPath], previous) => {
      if (!open || previous?.[0]) return;
      addTask.resetTo({
        ...initialAddTaskDraft(downloads.config()),
        source: initialSource,
        torrentPath: initialTorrentPath ?? "",
      });
      setAddError("");
      setAddNotice("");
    },
  );

  const chooseTorrent = (path: string) => {
    setSelectedTorrentFiles([]);
    setAddError("");
    if (untrack(torrentPath) === path) void torrentInspection.refresh();
    else setTorrentPath(path);
  };
  const torrentSummary = () => {
    if (inspectingTorrent()) return "Reading torrent metadata…";
    const preview = torrentPreview();
    if (preview)
      return `${preview.files.length} files · ${formatBytes(preview.totalLength)}`;
    return torrentPath() || "Inspect files before creating the task";
  };
  const pasteLinks = async () => {
    setAddError("");
    setAddNotice("");
    try {
      const text = await clipboard.readText();
      if (!text?.trim()) {
        setAddNotice("The clipboard does not contain text.");
        return;
      }
      const curl = parseCurlDownload(text);
      if (!curl) {
        setUrl(text.trim());
        setAddNotice("Pasted clipboard text.");
        return;
      }
      setUrl(curl.urls.join("\n"));
      setHeaders(curl.headers.join("\n"));
      setFilename(curl.output ?? "");
      setAddNotice(
        curl.proxy
          ? "Imported URLs and headers. Per-task proxies are not supported."
          : "Imported URLs and request headers from cURL.",
      );
    } catch (error) {
      setAddError(`Cannot read clipboard: ${String(error)}`);
    }
  };
  const createTask = async () => {
    if (!addTask.valid()) return;
    setAddError("");
    try {
      const options = {
        dir: directory().trim() || undefined,
        split: Number.parseInt(split(), 10) || undefined,
        priority: priority(),
      };
      if (source() === "torrent")
        await downloads.addTorrent({
          path: torrentPath(),
          selectedFiles: selectedTorrentFiles(),
          ...options,
        });
      else
        await downloads.addUris({
          uris: downloadUris(url()),
          out: filename().trim() || undefined,
          headers: headers()
            .split(/\r?\n/)
            .map((value) => value.trim())
            .filter(Boolean),
          ...options,
        });
      props.onOpenChange(false);
      await props.onCreated();
    } catch (error) {
      setAddError(String(error));
    }
  };

  return (
    <Modal
      aria-label="Add download task"
      open={props.open}
      onOpenChange={props.onOpenChange}
      contentClass="w-[560px] max-w-full max-h-11/12 overflow-hidden p-6 flex flex-col gap-4 rounded-xl border border-subtle bg-surface shadow-xl"
    >
      {({ close }) => (
        <>
          <Text class="text-xl font-semibold">Add download task</Text>
          <View class="flex gap-2">
            <Button
              size="sm"
              variant={source() === "links" ? "default" : "ghost"}
              onClick={() => setSource("links")}
            >
              Links
            </Button>
            <Button
              size="sm"
              variant={source() === "torrent" ? "default" : "ghost"}
              onClick={() => setSource("torrent")}
            >
              Torrent file
            </Button>
          </View>
          <DialogScrollBody contentClass="pr-2 flex flex-col gap-4">
            <Show
              when={source() === "links"}
              fallback={
                <View class="flex flex-col gap-3">
                  <View class="min-h-24 p-4 flex items-center justify-between gap-4 rounded-lg border border-strong bg-control">
                    <View class="min-w-0 flex-1 flex flex-col gap-1">
                      <Text class="truncate text-sm font-medium">
                        {torrentPreview()?.name ||
                          torrentPath() ||
                          "Choose a .torrent file"}
                      </Text>
                      <Text
                        role="status"
                        aria-label={torrentSummary()}
                        class="truncate text-xs text-muted"
                      >
                        {torrentSummary()}
                      </Text>
                    </View>
                    <Button
                      variant="outline"
                      onClick={async () => {
                        const paths = await dialog.open({
                          title: "Choose torrent",
                          filters: [
                            { name: "BitTorrent", extensions: ["torrent"] },
                          ],
                        });
                        if (paths?.[0]) chooseTorrent(paths[0]);
                      }}
                    >
                      Browse…
                    </Button>
                  </View>
                  <Show when={torrentPreview()} keyed>
                    {(preview) => (
                      <View class="flex flex-col gap-2">
                        <View class="flex items-center justify-between">
                          <Checkbox
                            label="Select all files"
                            checked={
                              selectedTorrentFiles().length ===
                              preview.files.length
                            }
                            indeterminate={
                              selectedTorrentFiles().length > 0 &&
                              selectedTorrentFiles().length <
                                preview.files.length
                            }
                            onCheckedChange={(checked) =>
                              setSelectedTorrentFiles(
                                checked
                                  ? preview.files.map((file) => file.index)
                                  : [],
                              )
                            }
                          />
                          <Text class="text-xs text-muted">
                            {selectedTorrentFiles().length} of{" "}
                            {preview.files.length}
                          </Text>
                        </View>
                        <View class="h-44 overflow-hidden rounded-lg border border-subtle bg-surface">
                          <VirtualList
                            items={() => preview.files}
                            getItemKey={(file) => file.index}
                            itemHeight={32}
                            viewportHeight={176}
                            accessibilityLabel="Torrent files"
                          >
                            {(file) => (
                              <View class="h-8 px-2 min-w-0 flex items-center gap-2">
                                <Checkbox
                                  class="min-w-0 flex-1"
                                  label={file().path}
                                  checked={selectedTorrentFiles().includes(
                                    file().index,
                                  )}
                                  onCheckedChange={(checked) =>
                                    setSelectedTorrentFiles((indices) =>
                                      checked
                                        ? [...indices, file().index].sort(
                                            (left, right) => left - right,
                                          )
                                        : indices.filter(
                                            (index) => index !== file().index,
                                          ),
                                    )
                                  }
                                />
                                <Text class="flex-none text-xs text-muted">
                                  {formatBytes(file().length)}
                                </Text>
                              </View>
                            )}
                          </VirtualList>
                        </View>
                      </View>
                    )}
                  </Show>
                </View>
              }
            >
              <View class="flex items-center justify-between gap-3">
                <Text class="text-sm text-muted">
                  Enter one HTTP, HTTPS or magnet link per line.
                </Text>
                <Button size="sm" variant="outline" onClick={pasteLinks}>
                  Paste
                </Button>
              </View>
              <TextArea
                class="h-28"
                aria-label="Download URLs"
                value={url()}
                placeholder="https://example.com/file.iso"
                onInput={(event) => setUrl(event.currentTarget.value)}
              />
              <Show when={addTask.fieldError("url")}>
                {(error) => (
                  <Text
                    role="alert"
                    aria-label="Download URI validation error"
                    class="text-sm text-danger-primary"
                  >
                    {error()}
                  </Text>
                )}
              </Show>
            </Show>
            <View class="flex gap-3">
              <DirectoryPicker
                class="min-w-0 flex-1"
                aria-label="Save directory"
                value={directory()}
                placeholder="Default download folder"
                browseLabel="Browse"
                browseAriaLabel="Browse save directory"
                dialogOptions={{ title: "Choose a download folder" }}
                onValueChange={setDirectory}
                onBrowseError={(error) => setAddError(String(error))}
              />
              <Input
                class="w-28"
                aria-label="Split count"
                value={split()}
                placeholder="16"
                onInput={(event) => setSplit(event.currentTarget.value)}
              />
            </View>
            <View class="flex flex-col gap-1">
              <Text class="text-xs text-muted">Queue priority</Text>
              <ToggleGroup
                type="single"
                aria-label="New task priority"
                value={priority()}
                class="w-full gap-1 bg-transparent p-0"
                onValueChange={(value) => {
                  if (value) setPriority(value as TaskPriority);
                }}
              >
                {(["low", "normal", "high", "critical"] as const).map(
                  (value) => (
                    <ToggleGroupItem value={value} class="min-w-0 flex-1">
                      {value[0].toUpperCase() + value.slice(1)}
                    </ToggleGroupItem>
                  ),
                )}
              </ToggleGroup>
            </View>
            <Show when={source() === "links"}>
              <Input
                aria-label="Output filename"
                value={filename()}
                placeholder="File name (automatic)"
                onInput={(event) => setFilename(event.currentTarget.value)}
              />
              <Show when={addTask.fieldError("filename")}>
                <Text role="alert" class="text-sm text-danger-primary">
                  {addTask.fieldError("filename")}
                </Text>
              </Show>
              <Collapsible class="rounded-lg border border-subtle p-3">
                <CollapsibleTrigger>
                  <Text class="text-sm font-medium">Advanced HTTP options</Text>
                </CollapsibleTrigger>
                <CollapsibleContent class="pt-3">
                  <View class="flex flex-col gap-3">
                    <TextArea
                      class="h-20"
                      aria-label="HTTP request headers"
                      value={headers()}
                      placeholder="Referer: https://example.com/"
                      onInput={(event) => setHeaders(event.currentTarget.value)}
                    />
                    <Text class="text-xs text-muted">
                      Put one Name: value header on each line.
                    </Text>
                  </View>
                </CollapsibleContent>
              </Collapsible>
            </Show>
            <Show
              when={
                addTask.fieldError("split") ?? addTask.fieldError("headers")
              }
            >
              {(error) => (
                <Text
                  role="alert"
                  aria-label="Add task validation error"
                  class="text-sm text-danger-primary"
                >
                  {error()}
                </Text>
              )}
            </Show>
            <Show when={addError()}>
              <Text
                role="alert"
                aria-label="Add task error"
                class="text-sm text-danger-primary"
              >
                {addError()}
              </Text>
            </Show>
            <Show when={addNotice()}>
              <Text role="status" class="text-sm text-muted">
                {addNotice()}
              </Text>
            </Show>
          </DialogScrollBody>
          <View class="flex justify-end gap-2">
            <Button variant="ghost" onClick={close}>
              Cancel
            </Button>
            <Button
              disabled={
                !addTask.valid() ||
                (source() === "links"
                  ? !url().trim()
                  : inspectingTorrent() ||
                    !torrentPreview() ||
                    selectedTorrentFiles().length === 0)
              }
              onClick={createTask}
            >
              Create task
            </Button>
          </View>
        </>
      )}
    </Modal>
  );
}
