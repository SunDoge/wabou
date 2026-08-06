// URL / URLSearchParams. Backed by `whatwg-url` — the reference implementation
// (same as Node.js), spec-accurate and complete. Exports as ESM so we install
// onto globalThis explicitly.

import { URL, URLSearchParams } from "whatwg-url";

(globalThis as any).URL = URL;
(globalThis as any).URLSearchParams = URLSearchParams;
