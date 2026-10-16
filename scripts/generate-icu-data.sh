#!/usr/bin/env sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
datagen=${ICU4X_DATAGEN:-icu4x-datagen}

"$datagen" \
  --cldr-tag 48.2.1 \
  --unicode-tag 17.0.0 \
  --markers \
    DecimalDigitsV1 \
    DecimalSymbolsV1 \
    PluralsCardinalV1 \
    PluralsOrdinalV1 \
    CalendarWeekV1 \
    LocaleLikelySubtagsLanguageV1 \
    LocaleParentsV1 \
    DatetimeNamesWeekdayV1 \
    DatetimeNamesYearGregorianV1 \
    DatetimeNamesMonthGregorianV1 \
    DatetimePatternsDateGregorianV1 \
    CurrencyEssentialsV1 \
    CurrencySymbolsV1 \
    CurrencyFractionsV1 \
    PercentEssentialsV1 \
    LongSecondRelativeV1 \
    LongMinuteRelativeV1 \
    LongHourRelativeV1 \
    LongDayRelativeV1 \
    LongWeekRelativeV1 \
    LongMonthRelativeV1 \
    LongQuarterRelativeV1 \
    LongYearRelativeV1 \
  --locales en en-US de de-DE zh zh-CN zh-TW \
  --format blob \
  --out "$repo_root/crates/wabou-runtime/data/icu4x.postcard" \
  --overwrite
