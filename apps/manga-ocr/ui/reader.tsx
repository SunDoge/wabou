import {
  AnnotationLayer,
  type AnnotationRegion,
  Badge,
  Button,
  Card,
  CardContent,
  createWindowMatch,
  type Handle,
  ImageList,
  ImageOverlayLayer,
  ImageViewport,
  NumberField,
  Progress,
  ScrollArea,
  Text,
  TextArea,
  View,
} from "@wabou/ui";
import imagesIcon from "lucide-static/icons/images.svg?raw";
import languages from "lucide-static/icons/languages.svg?raw";
import scanText from "lucide-static/icons/scan-text.svg?raw";
import hand from "lucide-static/icons/hand.svg?raw";
import squareDashed from "lucide-static/icons/square-dashed.svg?raw";
import { For, Show } from "solid-js";
import { Icon } from "@wabou/ui";
import { ocrStateLabel } from "./ocr-queue";
import {
  selectRegionAndReveal,
  translatedRegions,
  updateRegionGeometry,
} from "./reader-state";
import { useMangaSession } from "./session";

export function Reader() {
  const compact = createWindowMatch({ maxWidth: 1100 });
  const session = useMangaSession();
  const {
    pages,
    pageIndex,
    selectPage,
    setRegionsByPage,
    ocrStateByPage,
    ocrCompleted,
    selectedRegion,
    setSelectedRegion,
    zoom,
    setZoom,
    pan,
    setPan,
    tool,
    setTool,
    showBoxes,
    setShowBoxes,
    showOcr,
    setShowOcr,
    showTranslation,
    setShowTranslation,
    status,
    currentPage,
    regions,
    selected,
    operation,
    busy,
    runOcr,
    translate,
    adjustBboxes,
  } = session;
  const regionItems = new Map<string, Handle>();

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
      <View class="flex-1 min-h-0 min-w-0 p-3 flex flex-row gap-3">
        <Show when={!compact()}>
          <View class="w-56 flex-none min-h-0 overflow-hidden border border-subtle bg-surface">
            <Show
              when={pages().length > 0}
              fallback={
                <View class="h-full flex items-center justify-center p-5">
                  <Text
                    maxLines={3}
                    class="w-full text-sm text-muted text-center"
                  >
                    Choose images to build the page strip.
                  </Text>
                </View>
              }
            >
              <ImageList
                items={pages}
                getItemKey={(page) => page.id}
                getResource={(page) => page.handle}
                getLabel={(page) => page.name}
                getDescription={(page) =>
                  `${ocrStateLabel(ocrStateByPage()[page.id])} · ${page.width} × ${page.height}`
                }
                selectedKey={currentPage()?.id}
                onSelectionChange={(_, index) => {
                  selectPage(index);
                  setSelectedRegion(null);
                  setPan({ x: 0, y: 0 });
                }}
                itemHeight={92}
                thumbnailWidth={48}
                thumbnailHeight={68}
                accessibilityLabel="Manga pages"
                class="w-full h-full"
              />
            </Show>
          </View>
        </Show>

        <View class="flex-1 min-w-0 min-h-0 flex flex-col gap-2">
          <View class="min-h-10 flex-none px-2 py-1 flex flex-row flex-wrap items-center gap-2 rounded-lg border border-subtle bg-surface">
            <Button
              size="sm"
              variant="outline"
              disabled={operation.pending()}
              onClick={() => void session.openFiles()}
            >
              Open pages
            </Button>
            <Button
              size="sm"
              variant="ghost"
              disabled={operation.pending()}
              onClick={() => void session.openFolder()}
            >
              <Icon source={imagesIcon} size={15} /> Folder
            </Button>
            <View class="h-5 w-px bg-subtle" />
            <Button
              size="sm"
              variant={tool() === "pan" ? "secondary" : "ghost"}
              aria-label="Pan image"
              onClick={() => setTool("pan")}
            >
              <Icon source={hand} size={15} />
            </Button>
            <Button
              size="sm"
              variant={tool() === "regions" ? "secondary" : "ghost"}
              aria-label="Edit regions"
              onClick={() => setTool("regions")}
            >
              <Icon source={squareDashed} size={15} />
            </Button>
            <View class="h-5 w-px bg-subtle" />
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setZoom((value) => Math.max(0.25, value - 0.25))}
            >
              −
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                setZoom(1);
                setPan({ x: 0, y: 0 });
              }}
            >
              Fit {Math.round(zoom() * 100)}%
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setZoom((value) => Math.min(4, value + 0.25))}
            >
              +
            </Button>
            <Button
              size="sm"
              variant={showBoxes() ? "secondary" : "ghost"}
              onClick={() => setShowBoxes((value) => !value)}
            >
              Boxes
            </Button>
            <Button
              size="sm"
              variant={showOcr() ? "secondary" : "ghost"}
              onClick={() => setShowOcr((value) => !value)}
            >
              OCR
            </Button>
            <Button
              size="sm"
              variant={showTranslation() ? "secondary" : "ghost"}
              onClick={() => setShowTranslation((value) => !value)}
            >
              Translation
            </Button>
            <View class="flex-1" />
            <Show when={pages().length > 0}>
              <View class="w-28 flex-none flex flex-col gap-1">
                <Text class="text-xs text-muted">
                  OCR {ocrCompleted()}/{pages().length}
                </Text>
                <Progress
                  label="Background OCR progress"
                  value={ocrCompleted()}
                  maxValue={Math.max(1, pages().length)}
                  class="h-1"
                />
              </View>
            </Show>
            <Button
              size="sm"
              disabled={!currentPage() || operation.pending()}
              onClick={() => void runOcr()}
            >
              <Icon source={scanText} size={15} />
              {busy() === "ocr" ? "Recognizing…" : "Recognize again"}
            </Button>
          </View>
          <Show
            when={currentPage()}
            fallback={
              <View class="flex-1 min-h-0 rounded-xl border border-subtle bg-control flex items-center justify-center">
                <Text class="text-muted">Open a manga page to start.</Text>
              </View>
            }
          >
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
                  <ImageOverlayLayer
                    aria-label="OCR text overlay"
                    items={regions()}
                  >
                    {(region) => (
                      <Text
                        maxLines={3}
                        class="w-full h-full p-1 text-xs bg-surface text-primary"
                      >
                        {region.text}
                      </Text>
                    )}
                  </ImageOverlayLayer>
                </Show>
                <Show when={showTranslation()}>
                  <ImageOverlayLayer
                    aria-label="Translation overlay"
                    items={translatedRegions(regions())}
                  >
                    {(region) => (
                      <Text
                        maxLines={4}
                        class="w-full h-full p-1 text-xs bg-selected text-primary"
                      >
                        {region.translation}
                      </Text>
                    )}
                  </ImageOverlayLayer>
                </Show>
                <Show when={showBoxes()}>
                  <AnnotationLayer
                    aria-label="OCR regions"
                    regions={regions()}
                    selectedId={selectedRegion()}
                    interactionMode={
                      tool() === "regions" ? "edit" : "passthrough"
                    }
                    onSelectedIdChange={(id) =>
                      selectRegionAndReveal(id, setSelectedRegion, regionItems)
                    }
                    onRegionsChange={updateAnnotationGeometry}
                  />
                </Show>
              </ImageViewport>
            )}
          </Show>
          <Text class="h-5 flex-none text-xs text-muted">{status()}</Text>
        </View>

        <Card class="w-72 flex-none min-h-0 overflow-hidden">
          <CardContent class="h-full min-h-0 p-3 flex flex-col gap-3">
            <View class="flex flex-row items-center gap-2">
              <Text class="font-semibold">OCR & translation</Text>
            </View>
            <Button
              variant="secondary"
              disabled={regions().length === 0 || operation.pending()}
              onClick={() => void translate()}
            >
              <Icon source={languages} size={15} />
              {busy() === "translate" ? "Translating…" : "Translate all"}
            </Button>
            <Button
              variant="outline"
              disabled={regions().length === 0 || operation.pending()}
              onClick={() => void adjustBboxes()}
            >
              {busy() === "bbox" ? "Adjusting boxes…" : "Auto-adjust boxes"}
            </Button>
            <Show when={selected()}>
              {(region) => (
                <View class="grid grid-cols-2 gap-2 rounded-lg border border-subtle p-2">
                  <NumberField
                    aria-label="Region X"
                    value={Math.round(region().x)}
                    min={0}
                    max={currentPage()?.width ?? 0}
                    onValueChange={(value) =>
                      updateSelectedGeometry("x", value)
                    }
                  />
                  <NumberField
                    aria-label="Region Y"
                    value={Math.round(region().y)}
                    min={0}
                    max={currentPage()?.height ?? 0}
                    onValueChange={(value) =>
                      updateSelectedGeometry("y", value)
                    }
                  />
                  <NumberField
                    aria-label="Region width"
                    value={Math.round(region().width)}
                    min={1}
                    max={currentPage()?.width ?? 1}
                    onValueChange={(value) =>
                      updateSelectedGeometry("width", value)
                    }
                  />
                  <NumberField
                    aria-label="Region height"
                    value={Math.round(region().height)}
                    min={1}
                    max={currentPage()?.height ?? 1}
                    onValueChange={(value) =>
                      updateSelectedGeometry("height", value)
                    }
                  />
                </View>
              )}
            </Show>
            <ScrollArea class="flex-1 min-h-0 pr-1">
              <View class="flex flex-col gap-2">
                <For each={regions()}>
                  {(region, index) => (
                    <View
                      ref={(node: Handle) => regionItems.set(region.id, node)}
                      role="button"
                      aria-label={`OCR region ${index() + 1}`}
                      aria-selected={selectedRegion() === region.id}
                      focusOrder={selectedRegion() === region.id ? 0 : -1}
                      class={
                        selectedRegion() === region.id
                          ? "p-3 gap-2 rounded-lg border border-accent bg-selected"
                          : "p-3 gap-2 rounded-lg border border-subtle bg-surface-muted"
                      }
                      onClick={() => setSelectedRegion(region.id)}
                    >
                      <View class="flex flex-row items-center gap-2">
                        <Badge variant="secondary">{index() + 1}</Badge>
                        <Text class="text-xs text-muted">
                          {Math.round(region.confidence * 100)}%
                        </Text>
                      </View>
                      <TextArea
                        value={region.text}
                        aria-label={`OCR text ${index() + 1}`}
                        class="min-h-16"
                        onInput={(event) =>
                          updateRegionText(region.id, event.currentTarget.value)
                        }
                      />
                      <Show when={region.translation}>
                        <Text class="text-sm text-secondary">
                          {region.translation}
                        </Text>
                      </Show>
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
