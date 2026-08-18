# Paraglide i18n experiment

This experiment checks whether Paraglide's compiled message functions fit
Wabou's embedded QuickJS runtime. It deliberately uses only the
`globalVariable` and `baseLocale` strategies: Web URL, cookie, local-storage,
middleware, and document reload behavior are outside Wabou's runtime model.

Generated sources and bundles are ignored; the compiler remains the source of
truth. The important properties are that the final IIFE has no Node or DOM dependency,
unused messages tree-shake, and locale selection can be connected to a Solid
signal. Rebuild it from this directory with:

```sh
bun x paraglide-js compile --project ./project.inlang --outdir ./generated \
  --strategy globalVariable baseLocale --is-server false \
  --emit-ts-declarations --no-emit-git-ignore --no-emit-prettier-ignore \
  --no-emit-readme
bun x vite build --config vite.config.ts
bun x vite build --config vite.baseline.config.ts
bun x vite build --config vite.wabou.config.ts
```

## Result

Paraglide works in Wabou's QuickJS runtime. The Wabou Vite build was evaluated
with `JsRuntime`, including English/Chinese selection and plural rules. Locale
changes use Solid 2's explicit `flush` boundary, and each generated message is
given the current locale instead of relying on Paraglide's Web-oriented global
locale strategy.

The small plain-Vite probe added 3,211 bytes raw / 1,313 bytes gzip over the
Solid-only baseline for two used messages and two locales. The deliberately
unused message was absent from the output. The production Wabou bundle is much
larger because it includes the existing FormatJS ECMA-402 polyfills and locale
data; Paraglide does not duplicate that data.

This is promising enough to use as Wabou's recommended compiled i18n layer,
but it should remain an experiment until locale context and persistence have a
small framework-level API.
