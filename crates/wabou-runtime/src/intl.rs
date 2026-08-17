use std::sync::OnceLock;

use wabou_host_api::CalendarDateInfo;

fn system_zone() -> &'static jiff::tz::TimeZone {
    static TIME_ZONE: OnceLock<jiff::tz::TimeZone> = OnceLock::new();
    TIME_ZONE.get_or_init(jiff::tz::TimeZone::system)
}

pub(crate) fn system_locale() -> String {
    static LOCALE: OnceLock<String> = OnceLock::new();
    LOCALE
        .get_or_init(|| {
            sys_locale::get_locales()
                .find(|locale| !matches!(locale.as_str(), "C" | "POSIX"))
                .unwrap_or_else(|| "en-US".to_owned())
        })
        .clone()
}

pub(crate) fn system_time_zone() -> String {
    system_zone().iana_name().unwrap_or("UTC").to_owned()
}

pub(crate) fn system_calendar_date() -> CalendarDateInfo {
    let date = jiff::Timestamp::now().to_zoned(system_zone().clone());
    CalendarDateInfo {
        year: i32::from(date.year()),
        month: date.month() as u8,
        day: date.day() as u8,
    }
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
}
