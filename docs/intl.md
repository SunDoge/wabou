# Internationalization

Wabou exposes a deliberately small native localization API instead of installing
the ECMA-402 `Intl` object in QuickJS. Rust uses ICU4X for locale-sensitive
numbers, currencies, plural rules, calendar week data, dates, and relative
time, while Jiff handles time zones. Formatter objects and locale data stay out
of the JavaScript heap.

The runtime does not enable ICU4X `compiled_data`. It embeds a generated
postcard provider containing only Wabou's required data markers and the
English, German, and Chinese locale families. The blob can be regenerated with
`scripts/generate-icu-data.sh`; adding another locale is a data change rather
than an implementation change. Unsupported native locales fail explicitly.
Currency, percent, and relative-time formatting currently use the pinned
`icu_experimental` component; the public Wabou API remains narrower than that
dependency so it can be upgraded without exposing ICU4X internals.

```ts
import { intl } from "@wabou/core/i18n";

intl.formatNumber(1234.5, { locale: "de-DE" });
intl.formatDate(Date.now(), {
  locale: "zh-CN",
  dateStyle: "medium",
  timeZone: "Asia/Shanghai",
});
intl.pluralCategory(2, { locale: "en" });
```

The supported options are Wabou types rather than `Intl.*Options`. Unsupported
options fail explicitly instead of silently approximating Web behavior.

Generated Paraglide `registry.js` modules are adapted by `@wabou/vite` to use
the same native functions. Browser previews use the browser's built-in `Intl`
as a fallback; native bundles do not contain FormatJS polyfills or locale data.
