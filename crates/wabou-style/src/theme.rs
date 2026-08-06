use std::sync::OnceLock;

use crate::model::Theme;
use crate::rules::{BASE_COLORS, COLOR_SCALES, COLOR_STOPS, SPACING};

impl Default for Theme {
    fn default() -> Self {
        Self {
            spacing: SPACING
                .iter()
                .map(|(name, value)| ((*name).to_owned(), *value))
                .collect(),
            colors: BASE_COLORS
                .iter()
                .map(|(name, value)| ((*name).to_owned(), *value))
                .chain(COLOR_SCALES.iter().flat_map(|(family, values)| {
                    COLOR_STOPS
                        .iter()
                        .zip(values)
                        .map(move |(stop, value)| (format!("{family}-{stop}"), *value))
                }))
                .collect(),
        }
    }
}

pub(crate) fn default_theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(Theme::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_color_families_have_every_standard_stop() {
        let theme = Theme::default();
        assert_eq!(
            theme.colors.len(),
            BASE_COLORS.len() + COLOR_SCALES.len() * COLOR_STOPS.len()
        );
        for (family, _) in COLOR_SCALES {
            for stop in COLOR_STOPS {
                assert!(
                    theme.colors.contains_key(&format!("{family}-{stop}")),
                    "missing {family}-{stop}"
                );
            }
        }
    }

    #[test]
    fn default_palette_includes_previously_missing_stops() {
        let theme = Theme::default();
        assert_eq!(theme.colors["red-400"], 0xf87171ff);
        assert_eq!(theme.colors["emerald-500"], 0x10b981ff);
        assert_eq!(theme.colors["sky-950"], 0x082f49ff);
    }
}
