import {
  AnnotationLayer,
  type AnnotationRegion,
  Badge,
  Button,
  Card,
  CardContent,
  ImageList,
  ImageViewport,
  Input,
  ScrollArea,
  Text,
  TextArea,
  View,
  releaseImageResource,
  dialog,
} from "@wabou/ui";
import folderOpen from "lucide-static/icons/folder-open.svg?raw";
import imagesIcon from "lucide-static/icons/images.svg?raw";
import languages from "lucide-static/icons/languages.svg?raw";
import scanText from "lucide-static/icons/scan-text.svg?raw";
import { createMemo, createSignal, For, onCleanup, Show } from "solid-js";
import { Icon } from "@wabou/ui";
import { type ImagePage, type OcrRegion, useMangaReaderApi } from "./api";

function errorText(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function Reader() {
  const api = useMangaReaderApi();
  const [pages, setPages] = createSignal<readonly ImagePage[]>([]);
  const [pageIndex, setPageIndex] = createSignal(0);
  const [regionsByPage, setRegionsByPage] = createSignal<
    Readonly<Record<string, readonly OcrRegion[]>>
  >({});
  const [selectedRegion, setSelectedRegion] = createSignal<string | null>(null);
  const [zoom, setZoom] = createSignal(1);
  const [busy, setBusy] = createSignal<"open" | "ocr" | "translate" | "model">();
  const [status, setStatus] = createSignal("Open manga pages or a directory to begin.");
  const [modelInstalled, setModelInstalled] = createSignal(false);
  const [apiKey, setApiKey] = createSignal("");
  const [model, setModel] = createSignal("google/gemini-2.5-flash-lite");
  const currentPage = createMemo(() => pages()[pageIndex()]);
  const regions = createMemo(() => {
    const page = currentPage();
    return page ? (regionsByPage()[page.id] ?? []) : [];
  });

  void api.modelStatus().then((value) => setModelInstalled(value.installed));

  const acceptPages = (next: readonly ImagePage[]) => {
    for (const page of pages()) void releaseImageResource(page.handle);
    setPages(next);
    setPageIndex(0);
    setSelectedRegion(null);
    setStatus(next.length ? `Loaded ${next.length} pages.` : "No supported images found.");
  };
  onCleanup(() => {
    for (const page of pages()) void releaseImageResource(page.handle);
  });

  const openFiles = async () => {
    const paths = await dialog.open({
      title: "Open manga pages",
      multiple: true,
      filters: [{ name: "Images", extensions: ["jpg", "jpeg", "png", "webp", "bmp", "gif"] }],
    });
    if (paths?.length) acceptPages(await api.describeImages(paths));
  };

  const openFolder = async () => {
    const directory = await dialog.pickDirectory({ title: "Open manga directory" });
    if (directory) acceptPages(await api.listImages(directory));
  };

  const run = async (kind: "open" | "ocr" | "translate" | "model", action: () => Promise<void>) => {
    if (busy()) return;
    setBusy(kind);
    try {
      await action();
    } catch (error) {
      setStatus(errorText(error));
    } finally {
      setBusy(undefined);
    }
  };

  const runOcr = () => run("ocr", async () => {
    const page = currentPage();
    if (!page) return;
    if (!modelInstalled()) {
      setStatus("Install the OCR model first.");
      return;
    }
    const next = await api.recognizePage(page.handle);
    setRegionsByPage((current) => ({ ...current, [page.id]: next }));
    setStatus(`Recognized ${next.length} text regions on ${page.name}.`);
  });

  const translate = () => run("translate", async () => {
    const page = currentPage();
    const current = regions();
    if (!page || current.length === 0) return;
    const translated = await api.translate({
      texts: current.map((region) => region.text),
      apiKey: apiKey(),
      model: model(),
      targetLanguage: "Simplified Chinese",
    });
    setRegionsByPage((all) => ({
      ...all,
      [page.id]: current.map((region, index) => ({ ...region, translation: translated[index] })),
    }));
    setStatus(`Translated ${translated.length} regions.`);
  });

  const updateAnnotationGeometry = (next: readonly AnnotationRegion[]) => {
    const page = currentPage();
    if (!page) return;
    const current = new Map(regions().map((region) => [region.id, region]));
    setRegionsByPage((all) => ({
      ...all,
      [page.id]: next.map((region) => ({
        ...current.get(region.id),
        ...region,
        text: current.get(region.id)?.text ?? "",
        confidence: current.get(region.id)?.confidence ?? 0,
      })),
    }));
  };

  const updateRegionText = (id: string, text: string) => {
    const page = currentPage();
    if (!page) return;
    setRegionsByPage((all) => ({
      ...all,
      [page.id]: regions().map((region) =>
        region.id === id ? { ...region, text } : region,
      ),
    }));
  };

  return (
    <View class="w-full h-full min-w-0 min-h-0 flex flex-col bg-canvas text-primary">
      <View class="h-14 flex-none px-5 flex flex-row items-center gap-3 border-b border-subtle bg-surface shadow-sm">
        <View class="w-8 h-8 rounded-lg bg-accent flex items-center justify-center">
          <Icon source={imagesIcon} size={18} class="text-on-accent" />
        </View>
        <View class="flex flex-col">
          <Text class="font-semibold">Manga OCR</Text>
          <Text class="text-xs text-muted">Rust images · editable regions · LLM translation</Text>
        </View>
        <View class="flex-1" />
        <Button size="sm" variant="outline" onClick={() => void run("open", openFiles)}>
          Open pages
        </Button>
        <Button size="sm" variant="secondary" onClick={() => void run("open", openFolder)}>
          <Icon source={folderOpen} size={15} />
          Folder
        </Button>
      </View>

      <View class="flex-1 min-h-0 min-w-0 p-3 flex flex-row gap-3">
        <Card class="w-56 flex-none min-h-0 overflow-hidden">
          <CardContent class="h-full min-h-0 p-2">
            <Show when={pages().length > 0} fallback={<View class="h-full items-center justify-center p-5"><Text maxLines={3} class="w-full text-sm text-muted text-center">Choose images to build the page strip.</Text></View>}>
              <ImageList
                items={pages}
                getItemKey={(page) => page.id}
                getResource={(page) => page.handle}
                getLabel={(page) => page.name}
                getDescription={(page) => `${page.width} × ${page.height}`}
                selectedKey={currentPage()?.id}
                onSelectionChange={(_, index) => { setPageIndex(index); setSelectedRegion(null); }}
                itemHeight={92}
                thumbnailWidth={48}
                thumbnailHeight={68}
                accessibilityLabel="Manga pages"
                class="w-full h-full"
              />
            </Show>
          </CardContent>
        </Card>

        <View class="flex-1 min-w-0 min-h-0 flex flex-col gap-2">
          <View class="h-10 flex-none px-2 flex flex-row items-center gap-2 rounded-lg border border-subtle bg-surface">
            <Button size="sm" variant="ghost" onClick={() => setZoom((value) => Math.max(0.25, value - 0.25))}>−</Button>
            <Badge variant="secondary">{Math.round(zoom() * 100)}%</Badge>
            <Button size="sm" variant="ghost" onClick={() => setZoom((value) => Math.min(4, value + 0.25))}>+</Button>
            <View class="flex-1" />
            <Button size="sm" disabled={!currentPage() || busy() !== undefined} onClick={() => void runOcr()}>
              <Icon source={scanText} size={15} />
              {busy() === "ocr" ? "Recognizing…" : "Recognize page"}
            </Button>
          </View>
          <Show when={currentPage()} fallback={<View class="flex-1 min-h-0 rounded-xl border border-subtle bg-control items-center justify-center"><Text class="text-muted">Open a manga page to start.</Text></View>}>
            {(page) => (
              <ImageViewport
                aria-label="Manga page viewport"
                class="flex-1 min-h-0 rounded-xl border border-subtle shadow-lg"
                resource={page().handle}
                imageSize={{ width: page().width, height: page().height }}
                zoom={zoom()}
              >
                <AnnotationLayer
                  aria-label="OCR regions"
                  regions={regions()}
                  selectedId={selectedRegion()}
                  onSelectedIdChange={setSelectedRegion}
                  onRegionsChange={updateAnnotationGeometry}
                />
              </ImageViewport>
            )}
          </Show>
          <Text class="h-5 flex-none text-xs text-muted">{status()}</Text>
        </View>

        <Card class="w-80 flex-none min-h-0 overflow-hidden">
          <CardContent class="h-full min-h-0 p-3 flex flex-col gap-3">
            <View class="flex flex-row items-center gap-2">
              <Text class="font-semibold">OCR & translation</Text>
              <View class="flex-1" />
              <Badge variant={modelInstalled() ? "success" : "secondary"}>{modelInstalled() ? "Model ready" : "Model missing"}</Badge>
            </View>
            <Show when={!modelInstalled()}>
              <Button variant="outline" disabled={busy() !== undefined} onClick={() => void run("model", async () => { const value = await api.downloadModel(); setModelInstalled(value.installed); setStatus("OCR model installed."); })}>
                {busy() === "model" ? "Downloading model…" : "Install PP-OCRv6 small"}
              </Button>
            </Show>
            <Input aria-label="OpenRouter API key" value={apiKey()} placeholder="OpenRouter API key" onInput={(event) => setApiKey(event.currentTarget.value)} />
            <Input aria-label="Translation model" value={model()} onInput={(event) => setModel(event.currentTarget.value)} />
            <Button variant="secondary" disabled={regions().length === 0 || busy() !== undefined} onClick={() => void translate()}>
              <Icon source={languages} size={15} />
              {busy() === "translate" ? "Translating…" : "Translate all"}
            </Button>
            <ScrollArea class="flex-1 min-h-0 pr-1">
              <View class="flex flex-col gap-2">
                <For each={regions()}>
                  {(region, index) => (
                    <View
                      role="button"
                      aria-label={`OCR region ${index() + 1}`}
                      class={selectedRegion() === region.id ? "p-3 gap-2 rounded-lg border border-accent bg-selected" : "p-3 gap-2 rounded-lg border border-subtle bg-surface-muted"}
                      onClick={() => setSelectedRegion(region.id)}
                    >
                      <View class="flex flex-row items-center gap-2">
                        <Badge variant="secondary">{index() + 1}</Badge>
                        <Text class="text-xs text-muted">{Math.round(region.confidence * 100)}%</Text>
                      </View>
                      <TextArea
                        value={region.text}
                        aria-label={`OCR text ${index() + 1}`}
                        class="min-h-16"
                        onInput={(event) => updateRegionText(region.id, event.currentTarget.value)}
                      />
                      <Show when={region.translation}><Text class="text-sm text-secondary">{region.translation}</Text></Show>
                    </View>
                  )}
                </For>
              </View>
            </ScrollArea>
          </CardContent>
        </Card>
      </View>
    </View>
  );
}
