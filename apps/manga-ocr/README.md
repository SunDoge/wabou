# Manga OCR · Wabou

A native manga reader experiment built from Wabou's `ImageList`,
`ImageViewport`, and `AnnotationLayer` components. It is based on the workflow
of `RawMangaReader`, without a WebView or browser-owned image objects.

## Run

```bash
wabou dev apps/manga-ocr
```

Open individual pages or a directory. Install the PP-OCRv6 small model from
the right panel, recognize the current page, then optionally enter an
OpenRouter API key and translate all recognized regions.

## Resource model

- Solid owns only page metadata, selection, zoom, annotations, and translated
  strings.
- Wabou's Rust runtime loads, decodes, caches, and paints file images.
- The OCR capability receives an image resource handle. A dedicated
  `manga-ocr-inference` `SerialWorker` owns and reuses the OCR engine, processing
  requests serially while QuickJS and the window event loop remain responsive.
  Decoded pixels never cross the JavaScript boundary.
- OpenRouter requests run in Rust so provider credentials and HTTP response
  parsing are outside the UI bundle's state machinery. A dedicated
  `manga-ocr-llm` `SerialWorker` serializes translation and vision requests.
- OCR, translation, and automatic bbox results are cached under the application
  cache directory through `PersistentJsonCache`, keyed by source content and
  the relevant model/options. API keys are never part of cache keys or values.
- The viewport supports fit-relative zoom, direct panning, editable image-space
  boxes, numeric geometry controls, and independently toggled OCR/translation
  overlays.
- Reader and architecture screens use Wabou's native data-router adapter, so
  navigation does not depend on browser history or DOM APIs.

The first version intentionally keeps OCR/translation calls on the low-frequency
JSON capability path. A future resource-handle API can let OCR and painting
share the exact same decoded Rust allocation without changing the component
API.
