import { clipboard, createWindow, currentWindow, useWindow } from "@wabou/core";
import { createElement, defaultHost, mount } from "@wabou/solid-renderer";

Object.assign(globalThis, {
  __wabou_test_host_api: {
    clipboard,
    createWindow,
    currentWindow,
    host: defaultHost,
    windowState: useWindow,
  },
});

mount(() => createElement("main") as never);
