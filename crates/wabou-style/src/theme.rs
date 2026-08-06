use std::sync::OnceLock;

use crate::model::Theme;
use crate::rules::{COLORS, SPACING};

impl Default for Theme {
    fn default() -> Self {
        Self {
            spacing: SPACING
                .iter()
                .map(|(name, value)| ((*name).to_owned(), *value))
                .collect(),
            colors: COLORS
                .iter()
                .map(|(name, value)| ((*name).to_owned(), *value))
                .collect(),
        }
    }
}

pub(crate) fn default_theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(Theme::default)
}
