import {
  clipboard,
  createWindow,
  currentWindow,
  dialog,
  notification,
  useWindow,
} from "@wabou/core";
import { createElement, defaultHost, mount } from "@wabou/solid-renderer";

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
});

mount(() => createElement("main") as never);
