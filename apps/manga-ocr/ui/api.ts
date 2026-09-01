import {
  bindJsonCapability,
  type Host,
  type NativeCapability,
  useHost,
} from "@wabou/ui";

export interface ImagePage {
  id: string;
  path: string;
  name: string;
  width: number;
  height: number;
  handle: { lo: number; hi: number };
}

export interface OcrRegion {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  text: string;
  confidence: number;
  translation?: string;
}

export interface OcrModelStatus {
  installed: boolean;
  ready: boolean;
  version: string;
}

export interface RecentEntry {
  kind: "file" | "directory";
  path: string;
  label: string;
}

export interface ModelDownloadProgress {
  state: "idle" | "downloading" | "verifying" | "complete" | "failed";
  downloadedBytes: number;
  totalBytes: number;
  currentFile?: string;
  error?: string;
}

interface MangaReaderCapability extends NativeCapability {
  __wabouCapabilityVersion: number;
  listImages(request: string): string | PromiseLike<string>;
  describeImages(request: string): string | PromiseLike<string>;
  modelStatus(): string | PromiseLike<string>;
  modelDownloadProgress(): string | PromiseLike<string>;
  recentEntries(): string | PromiseLike<string>;
  downloadModel(): string | PromiseLike<string>;
  recognizePage(request: string): string | PromiseLike<string>;
  translate(request: string): string | PromiseLike<string>;
  adjustBboxes(request: string): string | PromiseLike<string>;
}

interface MangaReaderHost extends Host {
  mangaReader: MangaReaderCapability;
}

export function useMangaReaderApi() {
  const host = useHost<MangaReaderHost>();
  const call = bindJsonCapability(host.mangaReader, {
    name: "mangaReader",
    version: 2,
  });
  return {
    listImages: (directory: string) =>
      call<ImagePage[]>("listImages", { directory }),
    describeImages: (paths: readonly string[]) =>
      call<ImagePage[]>("describeImages", { paths }),
    modelStatus: () => call<OcrModelStatus>("modelStatus"),
    modelDownloadProgress: () =>
      call<ModelDownloadProgress>("modelDownloadProgress"),
    recentEntries: () => call<RecentEntry[]>("recentEntries"),
    downloadModel: () => call<OcrModelStatus>("downloadModel"),
    recognizePage: (handle: ImagePage["handle"]) =>
      call<OcrRegion[]>("recognizePage", { handle }),
    translate: (options: {
      texts: readonly string[];
      apiKey: string;
      model: string;
      targetLanguage: string;
    }) => call<string[]>("translate", options),
    adjustBboxes: (options: {
      handle: ImagePage["handle"];
      regions: readonly OcrRegion[];
      apiKey: string;
      model: string;
    }) => call<OcrRegion[]>("adjustBboxes", options),
  };
}
