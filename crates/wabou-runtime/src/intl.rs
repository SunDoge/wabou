use std::sync::OnceLock;

use icu_calendar::{types::Weekday, week::WeekInformation};
use icu_locale::fallback::LocaleFallbacker;
use icu_locale_core::Locale;
use icu_provider_adapters::fallback::LocaleFallbackProvider;
use icu_provider_blob::BlobDataProvider;
use serde::Deserialize;
use wabou_host_api::CalendarDateInfo;
use writeable::Writeable;

type IcuProvider = LocaleFallbackProvider<BlobDataProvider>;

fn icu_provider() -> &'static IcuProvider {
    static PROVIDER: OnceLock<IcuProvider> = OnceLock::new();
    PROVIDER.get_or_init(|| {
        let provider =
            BlobDataProvider::try_new_from_static_blob(include_bytes!("../data/icu4x.postcard"))
                .expect("embedded ICU4X data must be valid");
        let fallbacker = LocaleFallbacker::try_new_with_buffer_provider(&provider)
            .expect("embedded ICU4X data must include locale fallback tables");
        LocaleFallbackProvider::new(provider, fallbacker)
    })
}

fn parse_locale(locale: &str) -> Result<Locale, String> {
    locale
        .parse()
        .map_err(|error| format!("invalid locale {locale:?}: {error}"))
}

fn system_zone() -> &'static jiff::tz::TimeZone {
    static TIME_ZONE: OnceLock<jiff::tz::TimeZone> = OnceLock::new();
    TIME_ZONE.get_or_init(jiff::tz::TimeZone::system)
}

pub fn system_locale() -> String {
    static LOCALE: OnceLock<String> = OnceLock::new();
    LOCALE
        .get_or_init(|| {
            sys_locale::get_locales()
                .find(|locale| !matches!(locale.as_str(), "C" | "POSIX"))
                .unwrap_or_else(|| "en-US".to_owned())
        })
        .clone()
}

pub fn system_time_zone() -> String {
    system_zone().iana_name().unwrap_or("UTC").to_owned()
}

pub fn system_calendar_date() -> CalendarDateInfo {
    let date = jiff::Timestamp::now().to_zoned(system_zone().clone());
    CalendarDateInfo {
        year: i32::from(date.year()),
        month: date.month() as u8,
        day: date.day() as u8,
    }
}

pub fn first_weekday(locale: &str) -> Result<u8, String> {
    let locale = parse_locale(locale)?;
    let info = WeekInformation::try_new_with_buffer_provider(icu_provider(), locale.into())
        .map_err(|error| format!("unsupported locale for week information: {error}"))?;
    Ok(match info.first_weekday {
        Weekday::Sunday => 0,
        Weekday::Monday => 1,
        Weekday::Tuesday => 2,
        Weekday::Wednesday => 3,
        Weekday::Thursday => 4,
        Weekday::Friday => 5,
        Weekday::Saturday => 6,
    })
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NumberFormatOptions {
    #[serde(default)]
    style: NumberStyle,
    currency: Option<Currency>,
    minimum_fraction_digits: Option<u8>,
    maximum_fraction_digits: Option<u8>,
    #[serde(default = "default_true")]
    use_grouping: bool,
}

const fn default_true() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum NumberStyle {
    #[default]
    Decimal,
    Percent,
    Currency,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum Currency {
    Usd,
    Eur,
    Gbp,
    Jpy,
    Cny,
}

impl Currency {
    const fn code(self) -> &'static str {
        match self {
            Self::Usd => "USD",
            Self::Eur => "EUR",
            Self::Gbp => "GBP",
            Self::Jpy => "JPY",
            Self::Cny => "CNY",
        }
    }
}

pub fn format_number(locale: &str, mut value: f64, options: &str) -> Result<String, String> {
    if !value.is_finite() {
        return Err("number must be finite".to_owned());
    }
    let locale = parse_locale(locale)?;
    let options: NumberFormatOptions = serde_json::from_str(options)
        .map_err(|error| format!("invalid number format options: {error}"))?;
    if matches!(options.style, NumberStyle::Percent) {
        value *= 100.0;
    }
    let default_digits = match (options.style, options.currency) {
        (NumberStyle::Percent, _) | (NumberStyle::Currency, Some(Currency::Jpy)) => 0,
        (NumberStyle::Currency, _) => 2,
        _ => 3,
    };
    let maximum = options.maximum_fraction_digits.unwrap_or(default_digits);
    let minimum = options.minimum_fraction_digits.unwrap_or(
        if matches!(options.style, NumberStyle::Currency) {
            maximum
        } else {
            0
        },
    );
    if minimum > maximum || maximum > 15 {
        return Err("fraction digits must satisfy 0 <= minimum <= maximum <= 15".to_owned());
    }

    let negative = value.is_sign_negative();
    let absolute = value.abs();
    let rendered = format!("{absolute:.precision$}", precision = usize::from(maximum));
    let (integer, mut fraction) = rendered
        .split_once('.')
        .map_or((rendered.as_str(), ""), |(integer, fraction)| {
            (integer, fraction)
        });
    while fraction.len() > usize::from(minimum) && fraction.ends_with('0') {
        fraction = &fraction[..fraction.len() - 1];
    }
    let mut ascii = integer.to_owned();
    if !fraction.is_empty() {
        ascii.push('.');
        ascii.push_str(fraction);
    }
    if negative {
        ascii.insert(0, '-');
    }
    let decimal: icu_decimal::input::Decimal = ascii
        .parse()
        .map_err(|error| format!("failed to represent number: {error}"))?;
    let mut formatter_options = icu_decimal::options::DecimalFormatterOptions::default();
    if !options.use_grouping {
        formatter_options.grouping_strategy = Some(icu_decimal::options::GroupingStrategy::Never);
    }
    if matches!(options.style, NumberStyle::Currency) {
        let currency = options.currency.unwrap_or(Currency::Usd);
        let currency_type = currency
            .code()
            .parse()
            .map_err(|error| format!("invalid currency code: {error}"))?;
        let formatter = icu_experimental::dimension::currency::formatter::CurrencyFormatter::try_new_symbol_with_buffer_provider(
            icu_provider(),
            locale.into(),
            currency_type,
            Default::default(),
        )
        .map_err(|error| format!("unsupported locale for currency formatting: {error}"))?;
        return Ok(formatter
            .format_fixed_decimal(&decimal)
            .write_to_string()
            .into_owned());
    }

    if matches!(options.style, NumberStyle::Percent) {
        let formatter = icu_experimental::dimension::percent::formatter::PercentFormatter::try_new_with_buffer_provider(
            icu_provider(),
            locale.into(),
            Default::default(),
        )
        .map_err(|error| format!("unsupported locale for percent formatting: {error}"))?;
        return Ok(formatter.format(&decimal).write_to_string().into_owned());
    }

    let formatter = icu_decimal::DecimalFormatter::try_new_with_buffer_provider(
        icu_provider(),
        locale.into(),
        formatter_options,
    )
    .map_err(|error| format!("unsupported locale for number formatting: {error}"))?;
    Ok(formatter.format(&decimal).write_to_string().into_owned())
}

pub fn parse_number(locale: &str, input: &str, options: &str) -> Result<f64, String> {
    let locale = parse_locale(locale)?;
    let options: NumberFormatOptions = serde_json::from_str(options)
        .map_err(|error| format!("invalid number format options: {error}"))?;
    let formatter = icu_decimal::DecimalFormatter::try_new_with_buffer_provider(
        icu_provider(),
        locale.into(),
        Default::default(),
    )
    .map_err(|error| format!("unsupported locale for number parsing: {error}"))?;
    let probe: icu_decimal::input::Decimal = "12345.6"
        .parse()
        .expect("static decimal probe must be valid");
    let probe = formatter.format(&probe).write_to_string().into_owned();
    let mut separators = probe
        .chars()
        .filter(|character| !character.is_ascii_digit());
    let grouping_separator = separators
        .next()
        .ok_or_else(|| "locale did not provide a grouping separator".to_owned())?;
    let decimal_separator = separators
        .next_back()
        .ok_or_else(|| "locale did not provide a decimal separator".to_owned())?;
    let mut normalized = String::with_capacity(input.len());
    for character in input.trim().chars() {
        match character {
            '0'..='9' | '-' | '+' => normalized.push(character),
            character if character == decimal_separator => normalized.push('.'),
            character if character == grouping_separator => {}
            ' ' | '\u{a0}' | '\u{202f}' | '%' | '$' | '€' | '£' | '¥' => {}
            character => return Err(format!("unsupported character {character:?} in number")),
        }
    }
    let mut value: f64 = normalized
        .parse()
        .map_err(|_| format!("invalid localized number {input:?}"))?;
    if matches!(options.style, NumberStyle::Percent) {
        value /= 100.0;
    }
    Ok(value)
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PluralKind {
    #[default]
    Cardinal,
    Ordinal,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluralOptions {
    #[serde(default)]
    r#type: PluralKind,
    #[serde(default)]
    offset: f64,
}

pub fn plural_category(locale: &str, value: f64, options: &str) -> Result<String, String> {
    if !value.is_finite() {
        return Err("plural input must be finite".to_owned());
    }
    let locale = parse_locale(locale)?;
    let options: PluralOptions = serde_json::from_str(options)
        .map_err(|error| format!("invalid plural options: {error}"))?;
    let value = value - options.offset;
    let mut rules_options = icu_plurals::PluralRulesOptions::default();
    rules_options.rule_type = Some(match options.r#type {
        PluralKind::Cardinal => icu_plurals::PluralRuleType::Cardinal,
        PluralKind::Ordinal => icu_plurals::PluralRuleType::Ordinal,
    });
    let rules = icu_plurals::PluralRules::try_new_with_buffer_provider(
        icu_provider(),
        locale.into(),
        rules_options,
    )
    .map_err(|error| format!("unsupported locale for plural rules: {error}"))?;
    let decimal = icu_decimal::input::Decimal::try_from_f64(
        value,
        icu_decimal::input::FloatPrecision::RoundTrip,
    )
    .map_err(|error| format!("failed to represent plural input: {error}"))?;
    Ok(match rules.category_for(&decimal) {
        icu_plurals::PluralCategory::Zero => "zero",
        icu_plurals::PluralCategory::One => "one",
        icu_plurals::PluralCategory::Two => "two",
        icu_plurals::PluralCategory::Few => "few",
        icu_plurals::PluralCategory::Many => "many",
        icu_plurals::PluralCategory::Other => "other",
    }
    .to_owned())
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DateStyle {
    Short,
    #[default]
    Medium,
    Long,
    Full,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DateFormatOptions {
    #[serde(default)]
    date_style: Option<DateStyle>,
    year: Option<String>,
    month: Option<String>,
    weekday: Option<String>,
    time_zone: Option<String>,
}

pub fn format_date(locale: &str, epoch_millis: f64, options: &str) -> Result<String, String> {
    if !epoch_millis.is_finite() || epoch_millis < i64::MIN as f64 || epoch_millis > i64::MAX as f64
    {
        return Err("date must be a finite Unix timestamp in milliseconds".to_owned());
    }
    let locale = parse_locale(locale)?;
    let options: DateFormatOptions = serde_json::from_str(options)
        .map_err(|error| format!("invalid date format options: {error}"))?;
    let zone = match options.time_zone.as_deref() {
        Some("UTC") => jiff::tz::TimeZone::UTC,
        Some(name) => jiff::tz::TimeZone::get(name).map_err(|error| error.to_string())?,
        None => system_zone().clone(),
    };
    let date = jiff::Timestamp::from_millisecond(epoch_millis as i64)
        .map_err(|error| error.to_string())?
        .to_zoned(zone);
    let input = icu_calendar::Date::try_new_gregorian(
        i32::from(date.year()),
        date.month() as u8,
        date.day() as u8,
    )
    .map_err(|error| error.to_string())?;
    let prefs = locale.into();

    macro_rules! format_with {
        ($field_set:expr) => {{
            let formatter = icu_datetime::FixedCalendarDateTimeFormatter::<
                icu_calendar::Gregorian,
                _,
            >::try_new_with_buffer_provider(
                icu_provider(), prefs, $field_set
            )
            .map_err(|error| format!("unsupported locale for date formatting: {error}"))?;
            Ok(formatter.format(&input).write_to_string().into_owned())
        }};
    }

    if options.weekday.as_deref() == Some("short")
        && options.year.is_none()
        && options.month.is_none()
    {
        format_with!(icu_datetime::fieldsets::E::short())
    } else if options.year.as_deref() == Some("numeric")
        && options.month.as_deref() == Some("long")
        && options.date_style.is_none()
    {
        format_with!(icu_datetime::fieldsets::YM::long())
    } else {
        match options.date_style.unwrap_or_default() {
            DateStyle::Short => format_with!(icu_datetime::fieldsets::YMD::short()),
            DateStyle::Medium => format_with!(icu_datetime::fieldsets::YMD::medium()),
            DateStyle::Long => format_with!(icu_datetime::fieldsets::YMD::long()),
            DateStyle::Full => format_with!(icu_datetime::fieldsets::YMDE::long()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RelativeNumeric {
    Auto,
    #[default]
    Always,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelativeTimeOptions {
    #[serde(default)]
    numeric: RelativeNumeric,
}

pub fn format_relative_time(
    locale: &str,
    value: f64,
    unit: &str,
    options: &str,
) -> Result<String, String> {
    if !value.is_finite() {
        return Err("relative-time input must be finite".to_owned());
    }
    let locale = parse_locale(locale)?;
    let options: RelativeTimeOptions = serde_json::from_str(options)
        .map_err(|error| format!("invalid relative-time options: {error}"))?;
    let singular = unit.trim_end_matches('s');
    if !matches!(
        singular,
        "year" | "quarter" | "month" | "week" | "day" | "hour" | "minute" | "second"
    ) {
        return Err(format!("unsupported relative-time unit {unit:?}"));
    }
    let mut formatter_options =
        icu_experimental::relativetime::RelativeTimeFormatterOptions::default();
    formatter_options.numeric = match options.numeric {
        RelativeNumeric::Auto => icu_experimental::relativetime::options::Numeric::Auto,
        RelativeNumeric::Always => icu_experimental::relativetime::options::Numeric::Always,
    };
    macro_rules! formatter {
        ($constructor:ident) => {
            icu_experimental::relativetime::RelativeTimeFormatter::$constructor(
                icu_provider(),
                locale.clone().into(),
                formatter_options,
            )
            .map_err(|error| format!("unsupported locale for relative-time formatting: {error}"))?
        };
    }
    let formatter = match singular {
        "year" => formatter!(try_new_long_year_with_buffer_provider),
        "quarter" => formatter!(try_new_long_quarter_with_buffer_provider),
        "month" => formatter!(try_new_long_month_with_buffer_provider),
        "week" => formatter!(try_new_long_week_with_buffer_provider),
        "day" => formatter!(try_new_long_day_with_buffer_provider),
        "hour" => formatter!(try_new_long_hour_with_buffer_provider),
        "minute" => formatter!(try_new_long_minute_with_buffer_provider),
        _ => formatter!(try_new_long_second_with_buffer_provider),
    };
    let decimal = icu_decimal::input::Decimal::try_from_f64(
        value,
        icu_decimal::input::FloatPrecision::RoundTrip,
    )
    .map_err(|error| format!("failed to represent relative-time input: {error}"))?;
    Ok(formatter.format(decimal).write_to_string().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_environment_values_are_available() {
        assert!(!system_locale().is_empty());
        assert!(!system_time_zone().is_empty());
        let today = system_calendar_date();
        assert!((1..=12).contains(&today.month));
        assert!((1..=31).contains(&today.day));
    }

    #[test]
    fn native_formatters_cover_the_ui_contract() {
        assert_eq!(format_number("en", 1234.5, "{}").unwrap(), "1,234.5");
        assert_eq!(parse_number("de", "1.234,5", "{}").unwrap(), 1234.5);
        assert_eq!(first_weekday("en-US").unwrap(), 0);
        assert_eq!(first_weekday("de-DE").unwrap(), 1);
        assert_eq!(
            format_number("de", 12.0, r#"{"style":"currency","currency":"EUR"}"#).unwrap(),
            "12,00\u{a0}€"
        );
        assert_eq!(
            format_number("de", 0.5, r#"{"style":"percent"}"#).unwrap(),
            "50\u{a0}%"
        );
        assert_eq!(plural_category("en", 1.0, "{}").unwrap(), "one");
        assert_eq!(
            plural_category("en", 2.0, r#"{"type":"ordinal"}"#).unwrap(),
            "two"
        );
        assert_eq!(plural_category("zh", 1.0, "{}").unwrap(), "other");
        let swiss = format_number("de-CH", 1234.5, "{}").unwrap();
        assert_eq!(parse_number("de-CH", &swiss, "{}").unwrap(), 1234.5);
        assert_eq!(
            format_relative_time("en", -1.0, "day", r#"{"numeric":"auto"}"#).unwrap(),
            "yesterday"
        );
        assert_eq!(
            format_date("en", 0.0, r#"{"dateStyle":"medium","timeZone":"UTC"}"#).unwrap(),
            "Jan 1, 1970"
        );
    }
}
