import {
  bindJsonCapability,
  type Host,
  type NativeJsonCapability,
  useHost,
} from "@wabou/ui";

export interface ImagePage {
  id: string;
  path: string;
  name: string;
  width: number;
  height: number;
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

interface MangaReaderCapability extends NativeJsonCapability {
  __wabouCapabilityVersion: number;
  listImages(request: string): string | PromiseLike<string>;
  describeImages(request: string): string | PromiseLike<string>;
  modelStatus(): string | PromiseLike<string>;
  downloadModel(): string | PromiseLike<string>;
  recognizePage(request: string): string | PromiseLike<string>;
  translate(request: string): string | PromiseLike<string>;
}

interface MangaReaderHost extends Host {
  mangaReader: MangaReaderCapability;
}

export function useMangaReaderApi() {
  const host = useHost<MangaReaderHost>();
  const call = bindJsonCapability(host.mangaReader, {
    name: "mangaReader",
    version: 1,
  });
  return {
    listImages: (directory: string) =>
      call<ImagePage[]>("listImages", { directory }),
    describeImages: (paths: readonly string[]) =>
      call<ImagePage[]>("describeImages", { paths }),
    modelStatus: () => call<OcrModelStatus>("modelStatus"),
    downloadModel: () => call<OcrModelStatus>("downloadModel"),
    recognizePage: (path: string) =>
      call<OcrRegion[]>("recognizePage", { path }),
    translate: (options: {
      texts: readonly string[];
      apiKey: string;
      model: string;
      targetLanguage: string;
    }) => call<string[]>("translate", options),
  };
}

