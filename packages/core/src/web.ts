// Optional Web compatibility surface. Native Wabou applications that do not
// use fetch or WHATWG streams should not pay for the complete stream polyfill.
// Rust still owns HTTP execution, connection pooling, TLS, and cancellation.

import "./polyfills/dom-exception";
import "./polyfills/abort-controller";
import "./polyfills/streams";
import "./polyfills/encoding-streams";
import "./polyfills/fetch";

export { WabouResponse } from "./polyfills/fetch";
