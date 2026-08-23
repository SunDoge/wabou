import { createAsyncAction, dialog, releaseImageResource } from "@wabou/ui";
import {
  createContext,
  createMemo,
  createSignal,
  onCleanup,
  type ParentProps,
  type Setter,
  useContext,
} from "solid-js";
import {
  type ImagePage,
  type ModelDownloadProgress,
  type OcrRegion,
  type RecentEntry,
  useMangaReaderApi,
} from "./api";
import { type OcrPageState, prioritizeAllPageIndices } from "./ocr-queue";

type Operation = "open" | "ocr" | "translate" | "bbox" | "model";

function writeSignal<T>(setter: Setter<T>, value: T) {
  setter(() => value);
}

function errorText(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function createMangaSession() {
  const api = useMangaReaderApi();
  const [pages, setPages] = createSignal<readonly ImagePage[]>([]);
  const [pageIndex, setPageIndex] = createSignal(0);
  const [regionsByPage, setRegionsByPage] = createSignal<
    Readonly<Record<string, readonly OcrRegion[]>>
  >({});
  const [ocrStateByPage, setOcrStateByPage] = createSignal<
    Readonly<Record<string, OcrPageState>>
  >({});
  const [selectedRegion, setSelectedRegion] = createSignal<string | null>(null);
  const [zoom, setZoom] = createSignal(1);
  const [pan, setPan] = createSignal({ x: 0, y: 0 });
  const [tool, setTool] = createSignal<"pan" | "regions">("pan");
  const [showBoxes, setShowBoxes] = createSignal(true);
  const [showOcr, setShowOcr] = createSignal(false);
  const [showTranslation, setShowTranslation] = createSignal(true);
  const [status, setStatus] = createSignal(
    "Open manga pages or a directory to begin.",
  );
  const [modelInstalled, setModelInstalled] = createSignal(false);
  const [modelVersion, setModelVersion] = createSignal("");
  const [downloadProgress, setDownloadProgress] =
    createSignal<ModelDownloadProgress>({
      state: "idle",
      downloadedBytes: 0,
      totalBytes: 0,
    });
  const [recentEntries, setRecentEntries] = createSignal<
    readonly RecentEntry[]
  >([]);
  const [apiKey, setApiKey] = createSignal("");
  const [model, setModel] = createSignal("openrouter/free");
  const [theme, setTheme] = createSignal<"light" | "dark">("light");
  const currentPage = createMemo(() => pages()[pageIndex()]);
  const regions = createMemo(() => {
    const page = currentPage();
    return page ? (regionsByPage()[page.id] ?? []) : [];
  });
  const selected = createMemo(() =>
    regions().find((region) => region.id === selectedRegion()),
  );
  const ocrCompleted = createMemo(
    () =>
      Object.values(ocrStateByPage()).filter((state) => state === "complete")
        .length,
  );
  let ocrGeneration = 0;
  const operation = createAsyncAction(
    async (_kind: Operation, action: () => Promise<void>) => action(),
  );
  const busy = () => operation.pendingArgs()?.[0];

  const refreshModelStatus = async () => {
    const value = await api.modelStatus();
    setModelInstalled(value.installed);
    setModelVersion(value.version);
  };
  const refreshRecent = async () => setRecentEntries(await api.recentEntries());

  const recognizeInBackground = async (page: ImagePage, generation: number) => {
    setOcrStateByPage((states) => ({
      ...states,
      [page.id]: "recognizing",
    }));
    try {
      const next = await api.recognizePage(page.handle);
      if (generation !== ocrGeneration) return false;
      setRegionsByPage((current) => ({ ...current, [page.id]: next }));
      setOcrStateByPage((states) => ({
        ...states,
        [page.id]: "complete",
      }));
      return true;
    } catch (error) {
      if (generation === ocrGeneration) {
        setOcrStateByPage((states) => ({
          ...states,
          [page.id]: "failed",
        }));
        setStatus(`OCR failed for ${page.name}: ${errorText(error)}`);
      }
      return false;
    }
  };

  const scheduleBackgroundOcr = (centerIndex = pageIndex()) => {
    const currentPages = pages();
    const generation = ++ocrGeneration;
    if (!modelInstalled() || currentPages.length === 0) return;
    const candidates = prioritizeAllPageIndices(
      currentPages.length,
      centerIndex,
    )
      .map((index) => currentPages[index])
      .filter(
        (page): page is ImagePage =>
          page !== undefined && ocrStateByPage()[page.id] !== "complete",
      );
    setOcrStateByPage((states) => {
      const settled = Object.fromEntries(
        Object.entries(states).filter(
          ([, state]) => state === "complete" || state === "failed",
        ),
      );
      return {
        ...settled,
        ...Object.fromEntries(
          candidates.map((page) => [page.id, "queued" as const]),
        ),
      };
    });
    if (candidates.length === 0) return;
    void (async () => {
      let failures = 0;
      for (const page of candidates) {
        if (generation !== ocrGeneration) return;
        if (!(await recognizeInBackground(page, generation))) failures += 1;
      }
      if (generation !== ocrGeneration) return;
      setStatus(
        failures === 0
          ? "Background OCR completed for every page."
          : `Background OCR finished with ${failures} failed page${failures === 1 ? "" : "s"}.`,
      );
    })();
  };

  const acceptPages = (next: readonly ImagePage[]) => {
    ocrGeneration += 1;
    for (const page of pages()) void releaseImageResource(page.handle);
    writeSignal(setPages, next);
    writeSignal(setPageIndex, 0);
    writeSignal(setPan, { x: 0, y: 0 });
    writeSignal(setSelectedRegion, null);
    writeSignal(setRegionsByPage, {});
    writeSignal(setOcrStateByPage, {});
    writeSignal(
      setStatus,
      next.length
        ? `Loaded ${next.length} pages.`
        : "No supported images found.",
    );
    scheduleBackgroundOcr(0);
  };

  onCleanup(() => {
    ocrGeneration += 1;
    for (const page of pages()) void releaseImageResource(page.handle);
  });

  const run = (kind: Operation, action: () => Promise<void>) =>
    operation.run(kind, action).then((result) => {
      if (!result.ok) setStatus(errorText(result.error));
      return result.ok;
    });

  const openFiles = () =>
    run("open", async () => {
      const paths = await dialog.open({
        title: "Open manga pages",
        multiple: true,
        filters: [
          {
            name: "Images",
            extensions: ["jpg", "jpeg", "png", "webp", "bmp", "gif"],
          },
        ],
      });
      if (paths?.length) {
        acceptPages(await api.describeImages(paths));
        await refreshRecent();
      }
    });

  const openFolder = () =>
    run("open", async () => {
      const directory = await dialog.pickDirectory({
        title: "Open manga directory",
      });
      if (directory) {
        acceptPages(await api.listImages(directory));
        await refreshRecent();
      }
    });

  const openRecent = (entry: RecentEntry) =>
    run("open", async () => {
      const next =
        entry.kind === "directory"
          ? await api.listImages(entry.path)
          : await api.describeImages(entry.path.split("\n"));
      acceptPages(next);
      await refreshRecent();
    });

  const runOcr = () =>
    run("ocr", async () => {
      const page = currentPage();
      if (!page) return;
      if (!modelInstalled()) throw new Error("Install the OCR model first.");
      const generation = ++ocrGeneration;
      const completed = await recognizeInBackground(page, generation);
      if (!completed) return;
      setStatus(
        `Recognized ${regionsByPage()[page.id]?.length ?? 0} text regions on ${page.name}.`,
      );
      scheduleBackgroundOcr(pageIndex());
    });

  const translate = () =>
    run("translate", async () => {
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
        [page.id]: current.map((region, index) => ({
          ...region,
          translation: translated[index],
        })),
      }));
      setStatus(`Translated ${translated.length} regions.`);
    });

  const adjustBboxes = () =>
    run("bbox", async () => {
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

  const downloadModel = () =>
    run("model", async () => {
      let polling = true;
      const poll = async () => {
        while (polling) {
          setDownloadProgress(await api.modelDownloadProgress());
          await new Promise((resolve) => setTimeout(resolve, 150));
        }
      };
      const pollingTask = poll();
      const value = await api.downloadModel().finally(() => {
        polling = false;
      });
      await pollingTask;
      setDownloadProgress(await api.modelDownloadProgress());
      setModelInstalled(value.installed);
      setModelVersion(value.version);
      setStatus("OCR model installed.");
      scheduleBackgroundOcr();
    });

  const selectPage = (index: number) => {
    setPageIndex(index);
    scheduleBackgroundOcr(index);
  };

  void refreshModelStatus().then(() => scheduleBackgroundOcr());
  void refreshRecent();

  return {
    pages,
    pageIndex,
    selectPage,
    regionsByPage,
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
    setStatus,
    modelInstalled,
    modelVersion,
    downloadProgress,
    recentEntries,
    apiKey,
    setApiKey,
    model,
    setModel,
    theme,
    setTheme,
    currentPage,
    regions,
    selected,
    operation,
    busy,
    openFiles,
    openFolder,
    openRecent,
    runOcr,
    translate,
    adjustBboxes,
    downloadModel,
  };
}

export type MangaSession = ReturnType<typeof createMangaSession>;
const MangaSessionContext = createContext<MangaSession>();

export function MangaSessionProvider(props: ParentProps) {
  const value = createMangaSession();
  return (
    <MangaSessionContext value={value}>{props.children}</MangaSessionContext>
  );
}

export function useMangaSession(): MangaSession {
  const value = useContext(MangaSessionContext);
  if (!value)
    throw new Error("useMangaSession must be used inside MangaSessionProvider");
  return value;
}
