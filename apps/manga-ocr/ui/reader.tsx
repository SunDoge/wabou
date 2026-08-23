import {
  AnnotationLayer,
  type AnnotationRegion,
  Badge,
  Button,
  Card,
  CardContent,
  ImageList,
  ImageOverlayLayer,
  ImageViewport,
  Input,
  NumberField,
  ScrollArea,
  Text,
  TextArea,
  View,
  releaseImageResource,
  dialog,
  useNavigate,
} from "@wabou/ui";
import folderOpen from "lucide-static/icons/folder-open.svg?raw";
import imagesIcon from "lucide-static/icons/images.svg?raw";
import languages from "lucide-static/icons/languages.svg?raw";
import scanText from "lucide-static/icons/scan-text.svg?raw";
import info from "lucide-static/icons/info.svg?raw";
import hand from "lucide-static/icons/hand.svg?raw";
import squareDashed from "lucide-static/icons/square-dashed.svg?raw";
import { createMemo, createSignal, For, onCleanup, Show } from "solid-js";
import { Icon } from "@wabou/ui";
import { type ImagePage, type OcrRegion, useMangaReaderApi } from "./api";
import { translatedRegions, updateRegionGeometry } from "./reader-state";

function errorText(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function Reader() {
  const navigate = useNavigate();
  const api = useMangaReaderApi();
  const [pages, setPages] = createSignal<readonly ImagePage[]>([]);
  const [pageIndex, setPageIndex] = createSignal(0);
  const [regionsByPage, setRegionsByPage] = createSignal<
    Readonly<Record<string, readonly OcrRegion[]>>
  >({});
  const [selectedRegion, setSelectedRegion] = createSignal<string | null>(null);
  const [zoom, setZoom] = createSignal(1);
  const [pan, setPan] = createSignal({ x: 0, y: 0 });
  const [tool, setTool] = createSignal<"pan" | "regions">("pan");
  const [showBoxes, setShowBoxes] = createSignal(true);
  const [showOcr, setShowOcr] = createSignal(false);
  const [showTranslation, setShowTranslation] = createSignal(true);
  const [busy, setBusy] = createSignal<"open" | "ocr" | "translate" | "bbox" | "model">();
  const [status, setStatus] = createSignal("Open manga pages or a directory to begin.");
  const [modelInstalled, setModelInstalled] = createSignal(false);
  const [apiKey, setApiKey] = createSignal("");
  const [model, setModel] = createSignal("openrouter/free");
  const currentPage = createMemo(() => pages()[pageIndex()]);
  const regions = createMemo(() => {
    const page = currentPage();
    return page ? (regionsByPage()[page.id] ?? []) : [];
  });
  const selected = createMemo(() =>
    regions().find((region) => region.id === selectedRegion()),
  );

  void api.modelStatus().then((value) => setModelInstalled(value.installed));

  const acceptPages = (next: readonly ImagePage[]) => {
    for (const page of pages()) void releaseImageResource(page.handle);
    setPages(next);
    setPageIndex(0);
    setPan({ x: 0, y: 0 });
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

  const run = async (kind: "open" | "ocr" | "translate" | "bbox" | "model", action: () => Promise<void>) => {
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

  const adjustBboxes = () => run("bbox", async () => {
    const page = currentPage();
    if (!page || regions().length === 0) return;
    const adjusted = await api.adjustBboxes({
      handle: page.handle,
      regions: regions(),
      apiKey: apiKey(),
      model: model(),
    });
    setRegionsByPage((all) => ({ ...all, [page.id]: adjusted }));
    setStatus(`Adjusted ${adjusted.length} regions with the vision model.`);
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

  const updateSelectedGeometry = (
    property: "x" | "y" | "width" | "height",
    value: number | null,
  ) => {
    const page = currentPage();
    const region = selected();
    if (!page || !region || value === null) return;
    setRegionsByPage((all) => ({
      ...all,
      [page.id]: regions().map((item) =>
        item.id === region.id
          ? updateRegionGeometry(item, property, value, {
              width: page.width,
              height: page.height,
            })
          : item,
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
        <Button size="sm" variant="ghost" onClick={() => void navigate({ to: "/about" })}>
          <Icon source={info} size={15} />
          About
        </Button>
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
                onSelectionChange={(_, index) => { setPageIndex(index); setSelectedRegion(null); setPan({ x: 0, y: 0 }); }}
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
            <Button size="sm" variant={tool() === "pan" ? "secondary" : "ghost"} aria-label="Pan image" onClick={() => setTool("pan")}>
              <Icon source={hand} size={15} />
            </Button>
            <Button size="sm" variant={tool() === "regions" ? "secondary" : "ghost"} aria-label="Edit regions" onClick={() => setTool("regions")}>
              <Icon source={squareDashed} size={15} />
            </Button>
            <View class="h-5 w-px bg-subtle" />
            <Button size="sm" variant="ghost" onClick={() => setZoom((value) => Math.max(0.25, value - 0.25))}>−</Button>
            <Button size="sm" variant="ghost" onClick={() => { setZoom(1); setPan({ x: 0, y: 0 }); }}>Fit {Math.round(zoom() * 100)}%</Button>
            <Button size="sm" variant="ghost" onClick={() => setZoom((value) => Math.min(4, value + 0.25))}>+</Button>
            <Button size="sm" variant={showBoxes() ? "secondary" : "ghost"} onClick={() => setShowBoxes((value) => !value)}>Boxes</Button>
            <Button size="sm" variant={showOcr() ? "secondary" : "ghost"} onClick={() => setShowOcr((value) => !value)}>OCR</Button>
            <Button size="sm" variant={showTranslation() ? "secondary" : "ghost"} onClick={() => setShowTranslation((value) => !value)}>Translation</Button>
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
                pan={pan()}
                pannable={tool() === "pan"}
                onPanChange={setPan}
              >
                <Show when={showOcr()}>
                  <ImageOverlayLayer aria-label="OCR text overlay" items={regions()}>
                    {(region) => <Text maxLines={3} class="w-full h-full p-1 text-xs bg-surface text-primary">{region.text}</Text>}
                  </ImageOverlayLayer>
                </Show>
                <Show when={showTranslation()}>
                  <ImageOverlayLayer aria-label="Translation overlay" items={translatedRegions(regions())}>
                    {(region) => <Text maxLines={4} class="w-full h-full p-1 text-xs bg-selected text-primary">{region.translation}</Text>}
                  </ImageOverlayLayer>
                </Show>
                <Show when={showBoxes()}>
                  <AnnotationLayer
                    aria-label="OCR regions"
                    regions={regions()}
                    selectedId={selectedRegion()}
                    interactionMode={tool() === "regions" ? "edit" : "passthrough"}
                    onSelectedIdChange={setSelectedRegion}
                    onRegionsChange={updateAnnotationGeometry}
                  />
                </Show>
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
            <Button variant="outline" disabled={regions().length === 0 || busy() !== undefined} onClick={() => void adjustBboxes()}>
              {busy() === "bbox" ? "Adjusting boxes…" : "Auto-adjust boxes"}
            </Button>
            <Show when={selected()}>
              {(region) => (
                <View class="grid grid-cols-2 gap-2 rounded-lg border border-subtle p-2">
                  <NumberField aria-label="Region X" value={Math.round(region().x)} min={0} max={currentPage()?.width ?? 0} onValueChange={(value) => updateSelectedGeometry("x", value)} />
                  <NumberField aria-label="Region Y" value={Math.round(region().y)} min={0} max={currentPage()?.height ?? 0} onValueChange={(value) => updateSelectedGeometry("y", value)} />
                  <NumberField aria-label="Region width" value={Math.round(region().width)} min={1} max={currentPage()?.width ?? 1} onValueChange={(value) => updateSelectedGeometry("width", value)} />
                  <NumberField aria-label="Region height" value={Math.round(region().height)} min={1} max={currentPage()?.height ?? 1} onValueChange={(value) => updateSelectedGeometry("height", value)} />
                </View>
              )}
            </Show>
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
