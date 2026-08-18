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
});

mount(() => createElement("main") as never);
