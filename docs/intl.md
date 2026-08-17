# Internationalization

Wabou installs standards-compatible FormatJS implementations for the Intl
constructors used by its components. Locale and time-zone data are selected at
build time rather than shipping every CLDR record in every application.

Configure data in `wabou.toml`:

```toml
[intl]
locales = ["en", "zh"]
time-zones = "golden"
```

`locales` names FormatJS locale-data modules. The default is `["en", "zh"]`.
Applications should explicitly list every locale they expose to users.

`time-zones = "golden"` uses FormatJS's compact recommended time-zone set and
is the default. Use `"all"` only when the application must format every IANA
zone, including historical aliases omitted from the golden set.

Wabou currently installs `Intl.getCanonicalLocales`, `Intl.Locale`,
`Intl.PluralRules`, `Intl.NumberFormat`, and `Intl.DateTimeFormat`. Other
ECMA-402 constructors remain absent unless the application installs them.
Wabou components feature-detect optional constructors such as `Intl.Collator`.

The Rust host supplies the operating system locale, IANA time-zone name, and
local calendar date. Formatting behavior stays in the standard JavaScript Intl
surface so libraries can use it without Wabou-specific adapters.
