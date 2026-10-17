# ICU4X runtime data

`icu4x.postcard` contains only the locale families and data markers used by
Wabou's native formatting API. It is generated with `icu4x-datagen 2.3.0`
built with its `unstable` feature:

```sh
cargo install icu4x-datagen --version 2.3.0 --locked --features unstable
scripts/generate-icu-data.sh
```

The CLDR and Unicode source versions are pinned in the script. Do not replace
this blob with ICU4X's `compiled_data`; doing so embeds every locale supplied by
the selected components and substantially increases the runtime binary.

