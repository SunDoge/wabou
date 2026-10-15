---
"@wabou/vite": patch
---

Resolve generated FormatJS imports from `@wabou/vite` and declare the
polyfills as package-owned dependencies, allowing isolated and vendored Wabou
applications to build without repeating internal Intl dependencies.
