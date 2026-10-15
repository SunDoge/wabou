import { expect, test } from "bun:test";
import { mount, writer } from "@wabou/core/renderer";
import { createComponent, createSignal, flush } from "solid-js";
import { PdfView } from "../dist/index.mjs";

test("PdfView sends a complete typed snapshot to the native widget", () => {
  const configs: string[] = [];
  const setWidgetConfig = writer.setWidgetConfig.bind(writer);
  writer.setWidgetConfig = (_id, json) => configs.push(json);
  let disposeMount: (() => void) | undefined;
  try {
    const [page, setPage] = createSignal(3);
    disposeMount = mount(() =>
      createComponent(PdfView, {
        path: "/books/sample.pdf",
        get page() {
          return page();
        },
        renderScale: 1.5,
        background: [248, 246, 240, 255],
      }),
    );
    flush();
    setPage(4);
    flush();
  } finally {
    disposeMount?.();
    writer.setWidgetConfig = setWidgetConfig;
  }

  expect(configs.map((json) => JSON.parse(json))).toEqual(
    [3, 4].map((page) => ({
      path: "/books/sample.pdf",
      page,
      renderScale: 1.5,
      background: [248, 246, 240, 255],
    })),
  );
});
