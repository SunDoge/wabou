import {
  AnnotationLayer,
  type AnnotationRegion,
  Badge,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  ImageList,
  ImageViewport,
  Text,
  View,
} from "@wabou/ui";
import { createSignal, For } from "solid-js";
import "virtual:wabou-stylesheet";

export function MangaPageMock() {
  return (
    <View class="w-full h-full relative bg-white overflow-hidden">
      <View
        class="absolute border-2 border-black bg-slate-100"
        style={{ left: "5%", top: "4%", width: "42%", height: "30%" }}
      >
        <View
          class="absolute rounded-full bg-slate-300"
          style={{ left: "15%", top: "18%", width: "34%", height: "46%" }}
        />
        <View
          class="absolute bg-slate-700"
          style={{ left: "8%", bottom: "8%", width: "78%", height: "8%" }}
        />
      </View>
      <View
        class="absolute border-2 border-black bg-slate-200"
        style={{ right: "5%", top: "4%", width: "42%", height: "45%" }}
      >
        <View
          class="absolute rounded-full bg-white border border-slate-400"
          style={{ right: "8%", top: "8%", width: "54%", height: "28%" }}
        />
        <View
          class="absolute bg-slate-500"
          style={{ left: "10%", bottom: "8%", width: "80%", height: "42%" }}
        />
      </View>
      <View
        class="absolute border-2 border-black bg-slate-100"
        style={{ left: "5%", bottom: "4%", width: "90%", height: "43%" }}
      >
        <View
          class="absolute bg-slate-300"
          style={{ left: "6%", top: "10%", width: "36%", height: "78%" }}
        />
        <View
          class="absolute rounded-full bg-white border border-slate-400"
          style={{ right: "8%", top: "12%", width: "42%", height: "34%" }}
        />
        <View
          class="absolute bg-slate-700"
          style={{ right: "8%", bottom: "12%", width: "46%", height: "16%" }}
        />
      </View>
    </View>
  );
}

const initialRegions: readonly AnnotationRegion[] = [
  {
    id: "speech-1",
    label: "Speech region 1",
    x: 430,
    y: 95,
    width: 260,
    height: 125,
  },
  {
    id: "speech-2",
    label: "Speech region 2",
    x: 430,
    y: 710,
    width: 245,
    height: 155,
  },
];

const demoPages = Array.from({ length: 24 }, (_, index) => ({
  id: index,
  title: `Page ${index + 1}`,
}));

export function ImageViewportPage() {
  const [regions, setRegions] = createSignal(initialRegions);
  const [selected, setSelected] = createSignal<string | null>(null);
  const [zoom, setZoom] = createSignal(1);
  const [selectedPage, setSelectedPage] = createSignal(0);
  let nextRegion = 1;
  return (
    <View class="w-full min-w-0 flex flex-col gap-5">
      <View class="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          variant="outline"
          onClick={() => setZoom((value) => Math.max(0.5, value - 0.25))}
        >
          Zoom out
        </Button>
        <Button
          size="sm"
          variant="outline"
          onClick={() => setZoom((value) => Math.min(3, value + 0.25))}
        >
          Zoom in
        </Button>
        <Button size="sm" variant="ghost" onClick={() => setZoom(1)}>
          Reset
        </Button>
        <Badge variant="secondary">{Math.round(zoom() * 100)}%</Badge>
        <Text class="text-xs text-muted">
          Drag empty page space to create a region. Drag a region or its corner
          handle to edit it.
        </Text>
      </View>
      <View class="w-full min-w-0 flex flex-row gap-4 items-start">
        <ImageList
          items={() => demoPages}
          getItemKey={(page) => page.id}
          getSource={(page) => ({
            kind: "file",
            path: `/demo/page-${page.id + 1}.png`,
          })}
          getLabel={(page) => page.title}
          renderThumbnail={() => <MangaPageMock />}
          selectedKey={selectedPage()}
          onSelectionChange={(page) => setSelectedPage(page.id)}
          itemHeight={88}
          thumbnailWidth={48}
          thumbnailHeight={68}
          accessibilityLabel="Manga pages"
          class="w-48 flex-none rounded-xl border border-subtle bg-surface"
          viewportHeight={620}
        />
        <ImageViewport
          aria-label="Manga annotation viewport"
          class="flex-1 min-w-0 rounded-xl border border-subtle shadow-sm"
          style={{ height: "620px" }}
          imageSize={{ width: 800, height: 1200 }}
          zoom={zoom()}
          media={<MangaPageMock />}
        >
          <AnnotationLayer
            aria-label="Manga OCR regions"
            regions={regions()}
            selectedId={selected()}
            createRegionId={() => `region-${nextRegion++}`}
            onSelectedIdChange={setSelected}
            onRegionsChange={setRegions}
          />
        </ImageViewport>
        <Card class="w-64 flex-none">
          <CardHeader>
            <CardTitle>OCR regions</CardTitle>
          </CardHeader>
          <CardContent class="gap-2">
            <For each={regions()}>
              {(region, index) => (
                <Button
                  size="sm"
                  variant={selected() === region.id ? "secondary" : "ghost"}
                  class="w-full justify-start"
                  onClick={() => setSelected(region.id)}
                >
                  {`${index() + 1}. ${region.label ?? region.id}`}
                </Button>
              )}
            </For>
          </CardContent>
        </Card>
      </View>
    </View>
  );
}
