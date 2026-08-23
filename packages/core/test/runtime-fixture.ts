import {
  clipboard,
  createWindow,
  currentWindow,
  dialog,
  notification,
  useWindow,
} from "@wabou/core";
import { createElement, defaultHost, mount } from "../src/renderer";

Object.assign(globalThis, {
  __wabou_test_host_api: {
    clipboard,
    dialog,
    notification,
    createWindow,
    currentWindow,
    host: defaultHost,
    windowState: useWindow,
  },
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
});

mount(() => createElement("main") as never);
