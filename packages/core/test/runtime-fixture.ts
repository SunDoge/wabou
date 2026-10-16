import {
  clipboard,
  createWindow,
  currentWindow,
  dialog,
  notification,
  useWindow,
} from "@wabou/core";
import "@wabou/core/web";
import { createEffect, createRoot, createSignal, flush, merge, omit } from "solid-js";
import {
  createElement,
  createTextNode,
  defaultHost,
  mount,
  spread,
} from "../src/renderer";

Object.assign(globalThis, {
  __wabou_test_solid: { createEffect, createSignal },
  __wabou_test_host_api: {
    clipboard,
    dialog,
    notification,
    createWindow,
    currentWindow,
    host: defaultHost,
    windowState: useWindow,
  },
  __wabou_test_proxy_children: () =>
    createRoot((dispose) => {
      const label = createTextNode("retained");
      const inner = createElement("view");
      spread(
        inner,
        omit(
          merge(
            {
              get children() {
                return label;
              },
            },
            { class: "inner" },
          ),
          "class",
        ),
        false,
      );
      const outer = createElement("view");
      spread(
        outer,
        merge(
          {
            get children() {
              return inner;
            },
          },
          { class: "outer" },
        ),
        false,
      );
      flush();
      const result = {
        mergedChild: outer.firstChild === inner,
        omittedChild: inner.firstChild === label,
      };
      dispose();
      return result;
    }),
  __wabou_test_streams: async () => {
    const output = new ReadableStream<string>({
      start(controller) {
        controller.enqueue("quick");
        controller.enqueue("js");
        controller.close();
      },
    }).pipeThrough(
      new TransformStream<string, string>({
        transform(chunk, controller) {
          controller.enqueue(chunk.toUpperCase());
        },
      }),
    );
    let text = "";
    for await (const chunk of output) text += chunk;
    return text;
  },
  __wabou_test_encoding_streams: async () => {
    const bytes = new TextEncoder().encode("漫画");
    const output = new ReadableStream<BufferSource>({
      start(controller) {
        controller.enqueue(bytes.slice(0, 2));
        controller.enqueue(bytes.slice(2));
        controller.close();
      },
    }).pipeThrough(new TextDecoderStream());
    let text = "";
    for await (const chunk of output) text += chunk;
    const response = new Response("buffered body");
    const reader = response.body?.getReader();
    const responseChunk = await reader?.read();
    return {
      text,
      responseText: new TextDecoder().decode(responseChunk?.value),
      bodyUsed: response.bodyUsed,
    };
  },
});

mount(() => createElement("main") as never);
