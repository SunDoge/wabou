// DOM-independent platform implementations. This bundle is evaluated only
// after bootstrap.ts has installed the codecs required by whatwg-url.

import structuredCloneImpl from "@ungap/structured-clone";
import { URL, URLSearchParams } from "whatwg-url";

const runtime = globalThis as Record<string, any>;

runtime.URL ??= URL;
runtime.URLSearchParams ??= URLSearchParams;
runtime.structuredClone ??= structuredCloneImpl;
