//! Retained host-side selection state. No plugin types or callbacks are retained.
//! The actor remains authoritative for the committed value; input returns requests.
pub use blockui_abi::select::OptionItem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Home,
    End,
    PageUp(usize),
    PageDown(usize),
    Activate,
    Escape,
}
#[derive(Default)]
pub struct SelectModel {
    options: Vec<OptionItem>,
    selected: Option<i64>,
    active: Option<i64>,
    open: bool,
    enabled: bool,
    prefix: String,
    prefix_at: Option<std::time::Instant>,
}
impl SelectModel {
    /// Copy borrowed implementation option bytes into host ownership. A failed
    /// decode or projection leaves the previously admitted control intact.
    pub fn project_wire(&mut self, wire: &str, selected: i64, enabled: bool) -> Result<(), String> {
        let options = blockui_abi::select::decode(wire).map_err(str::to_owned)?;
        self.project(options, selected, enabled)
    }

    /// Admit a complete projection before changing retained state. Values are
    /// business identities, so reordering or replacement preserves the active
    /// option. Removing it falls back to the committed or first enabled option.
    /// Projection never emits a change request.
    pub fn project(
        &mut self,
        options: Vec<OptionItem>,
        selected: i64,
        enabled: bool,
    ) -> Result<(), String> {
        blockui_abi::select::validate(&options).map_err(str::to_owned)?;
        if !options.iter().any(|option| option.value == selected) {
            return Err("select value is absent from options".into());
        }
        self.options = options;
        self.project_value(selected, enabled)
    }
    /// Reproject an already-admitted option set without allocating new labels.
    /// Invalid values leave the prior control state intact.
    pub fn project_value(&mut self, selected: i64, enabled: bool) -> Result<(), String> {
        if !self.options.iter().any(|item| item.value == selected) {
            return Err("select value is absent from options".into());
        }
        let available = |value| {
            self.options
                .iter()
                .any(|item| item.value == value && item.enabled)
        };
        let active = self
            .active
            .filter(|value| self.open && available(*value))
            .or_else(|| available(selected).then_some(selected))
            .or_else(|| {
                self.options
                    .iter()
                    .find(|item| item.enabled)
                    .map(|item| item.value)
            });
        self.selected = Some(selected);
        self.active = active;
        self.enabled = enabled;
        if !enabled || active.is_none() {
            self.close();
        }
        Ok(())
    }
    pub fn options(&self) -> &[OptionItem] {
        &self.options
    }
    pub fn selected(&self) -> Option<i64> {
        self.selected
    }
    pub fn active(&self) -> Option<i64> {
        self.active
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// Hover changes only the uncommitted active value of an open control.
    pub fn highlight(&mut self, value: i64) -> bool {
        if !self.enabled
            || !self.open
            || !self
                .options
                .iter()
                .any(|item| item.value == value && item.enabled)
            || self.active == Some(value)
        {
            return false;
        }
        self.active = Some(value);
        true
    }
    /// Case-insensitive Unicode lowercase prefix search with a 750ms timeout.
    /// Repeated single characters cycle enabled matches. At most 64 UTF8 bytes
    /// are retained. Open controls highlight only; closed controls request a
    /// selection and wait for actor projection to commit it.
    pub fn type_text(&mut self, text: &str, now: std::time::Instant) -> Option<i64> {
        if !self.enabled || text.is_empty() || text.len() > 64 || text.chars().any(char::is_control)
        {
            return None;
        }
        if self.prefix_at.is_none_or(|previous| {
            now.saturating_duration_since(previous) >= std::time::Duration::from_millis(750)
        }) {
            self.prefix.clear();
        }
        let cycle = self.prefix.chars().count() == 1
            && text.chars().count() == 1
            && self.prefix.to_lowercase() == text.to_lowercase();
        if !cycle {
            if self.prefix.len() + text.len() > 64 {
                self.prefix.clear();
            }
            self.prefix.push_str(text);
        }
        self.prefix_at = Some(now);
        let prefix = self.prefix.to_lowercase();
        let current = self
            .active
            .and_then(|value| self.options.iter().position(|option| option.value == value))
            .unwrap_or(0);
        let start = if cycle {
            (current + 1) % self.options.len()
        } else {
            current
        };
        let value = (0..self.options.len())
            .map(|offset| &self.options[(start + offset) % self.options.len()])
            .find(|option| option.enabled && option.label.to_lowercase().starts_with(&prefix))
            .map(|option| option.value)?;
        self.active = Some(value);
        (!self.open && self.selected != Some(value)).then_some(value)
    }
    pub fn close(&mut self) {
        self.open = false;
        self.prefix.clear();
        self.prefix_at = None;
        self.active = self
            .selected
            .filter(|value| {
                self.options
                    .iter()
                    .any(|item| item.value == *value && item.enabled)
            })
            .or_else(|| {
                self.options
                    .iter()
                    .find(|item| item.enabled)
                    .map(|item| item.value)
            });
    }
    /// Return an owned selection request. Do not update the committed value:
    /// the current actor action may reject it. Repeated activation is ignored.
    pub fn key(&mut self, key: Key, repeat: bool) -> Option<i64> {
        if !self.enabled {
            return None;
        }
        if key == Key::Escape {
            self.close();
            return None;
        }
        if key == Key::Activate {
            if repeat {
                return None;
            }
            if self.open {
                let request = self.active.filter(|value| Some(*value) != self.selected);
                self.close();
                return request;
            }
            self.open = self.active.is_some();
            return None;
        }
        self.prefix.clear();
        self.prefix_at = None;
        let mut available = self
            .options
            .iter()
            .filter(|item| item.enabled)
            .map(|item| item.value);
        let count = available.clone().count();
        if count == 0 {
            return None;
        }
        let index = self
            .active
            .and_then(|value| available.clone().position(|candidate| candidate == value))
            .unwrap_or(0);
        let next = match key {
            Key::Up => index.saturating_sub(1),
            Key::Down => (index + 1).min(count - 1),
            Key::Home => 0,
            Key::End => count - 1,
            Key::PageUp(rows) => index.saturating_sub(rows.clamp(1, 8)),
            Key::PageDown(rows) => index.saturating_add(rows.clamp(1, 8)).min(count - 1),
            _ => unreachable!(),
        };
        let value = available.nth(next).expect("bounded enabled option index");
        self.active = Some(value);
        (!self.open && self.active != self.selected).then_some(value)
    }
    /// Pointer input validates current options, including after replacement.
    pub fn choose(&mut self, value: i64) -> Option<i64> {
        if !self.enabled
            || !self.open
            || !self
                .options
                .iter()
                .any(|item| item.value == value && item.enabled)
        {
            return None;
        }
        self.active = Some(value);
        self.close();
        (Some(value) != self.selected).then_some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(value: i64, enabled: bool) -> OptionItem {
        OptionItem {
            value,
            enabled,
            label: format!("Option {value}"),
        }
    }
    #[test]
    fn cancellation_reopens_at_committed_value_but_confirmation_keeps_request() {
        let mut model = SelectModel::default();
        model
            .project(vec![item(1, true), item(2, true)], 1, true)
            .unwrap();
        model.key(Key::Activate, false);
        model.key(Key::Down, false);
        assert_eq!(model.active(), Some(2));
        model.key(Key::Escape, false);
        model.key(Key::Activate, false);
        assert_eq!(model.active(), Some(1));
        model.highlight(2);
        model.close(); // Blur/outside click/resize follows the same cancellation path.
        model.key(Key::Activate, false);
        assert_eq!(model.active(), Some(1));
        model.key(Key::Down, false);
        assert_eq!(model.key(Key::Activate, false), Some(2));
        assert_eq!(model.selected(), Some(1));
        assert_eq!(model.active(), Some(1)); // Actor has not accepted the request yet.
        model.project_value(2, true).unwrap();
        model.key(Key::Activate, false);
        assert_eq!(model.active(), Some(2));
    }
    #[test]
    fn all_disabled_options_never_open_or_publish_input() {
        let mut model = SelectModel::default();
        model
            .project(vec![item(1, false), item(2, false)], 1, true)
            .unwrap();
        for key in [
            Key::Activate,
            Key::Up,
            Key::Down,
            Key::Home,
            Key::End,
            Key::PageUp(8),
            Key::PageDown(8),
        ] {
            assert_eq!(model.key(key, false), None);
            assert!(!model.is_open());
        }
        assert_eq!(model.type_text("Option", std::time::Instant::now()), None);
        assert_eq!(model.choose(2), None);
        assert_eq!(model.active(), None);
        assert_eq!(model.selected(), Some(1));
        model
            .project(vec![item(1, false), item(2, true)], 1, true)
            .unwrap();
        model.key(Key::Activate, false);
        assert!(model.is_open());
        assert_eq!(model.active(), Some(2));
        assert_eq!(model.key(Key::Activate, false), Some(2));
    }
    #[test]
    fn replacement_preserves_business_identity_and_rejects_bad_projection() {
        let mut model = SelectModel::default();
        model
            .project(
                vec![item(10, true), item(20, false), item(30, true)],
                10,
                true,
            )
            .unwrap();
        assert_eq!(model.key(Key::Activate, false), None);
        assert_eq!(model.key(Key::Down, false), None);
        assert_eq!(model.active(), Some(30));
        assert!(
            model
                .project(vec![item(10, true), item(10, true)], 10, true)
                .is_err()
        );
        assert!(model.is_open());
        assert!(
            model
                .project_wire("select1\n30\t1\t1\n🦀", 30, true)
                .is_err()
        );
        assert_eq!(model.active(), Some(30));
        model
            .project(vec![item(30, true), item(10, true)], 10, true)
            .unwrap();
        assert_eq!(model.active(), Some(30));
        assert_eq!(model.choose(20), None);
        assert!(model.is_open());
        assert_eq!(model.key(Key::Activate, true), None);
        assert!(model.is_open());
        assert_eq!(model.key(Key::Activate, false), Some(30));
        assert_eq!(model.selected(), Some(10)); // request may still be rejected
        model
            .project(vec![item(30, true), item(10, true)], 10, true)
            .unwrap();
        assert_eq!(model.active(), Some(10));
    }
    #[test]
    fn timed_unicode_prefix_and_repeat_cycle_use_current_enabled_labels() {
        let now = std::time::Instant::now();
        let mut model = SelectModel::default();
        let options = vec![
            OptionItem {
                value: 1,
                label: "Åland".into(),
                enabled: true,
            },
            OptionItem {
                value: 2,
                label: "Ålesund".into(),
                enabled: false,
            },
            OptionItem {
                value: 3,
                label: "Århus".into(),
                enabled: true,
            },
            OptionItem {
                value: 4,
                label: "Berlin".into(),
                enabled: true,
            },
        ];
        model.project(options.clone(), 1, true).unwrap();
        model.key(Key::Activate, false);
        assert_eq!(model.type_text("å", now), None);
        assert_eq!(model.active(), Some(1));
        model.type_text("Å", now + std::time::Duration::from_millis(100));
        assert_eq!(model.active(), Some(3));
        model.type_text("B", now + std::time::Duration::from_millis(900));
        assert_eq!(model.active(), Some(4));
        assert_eq!(model.selected(), Some(1));
        assert_eq!(model.type_text("\n", now), None);
        assert_eq!(model.type_text(&"x".repeat(65), now), None);
        model.project(options, 1, true).unwrap();
        assert_eq!(model.active(), Some(4));
        assert_eq!(model.key(Key::Activate, false), Some(4));
        model
            .project(
                vec![
                    OptionItem {
                        value: 1,
                        label: "Alpha".into(),
                        enabled: true,
                    },
                    OptionItem {
                        value: 4,
                        label: "Renamed".into(),
                        enabled: true,
                    },
                ],
                1,
                true,
            )
            .unwrap();
        assert_eq!(
            model.type_text("R", now + std::time::Duration::from_secs(2)),
            Some(4)
        );
    }
    #[test]
    fn replacement_keeps_prefix_but_disable_clears_it() {
        let now = std::time::Instant::now();
        let mut model = SelectModel::default();
        let old = vec![
            OptionItem {
                value: 0,
                label: "One-way".into(),
                enabled: true,
            },
            OptionItem {
                value: 1,
                label: "Return".into(),
                enabled: true,
            },
        ];
        model.project(old, 0, true).unwrap();
        model.key(Key::Activate, false);
        model.type_text("Re", now);
        assert_eq!(model.active(), Some(1));
        assert!(model.highlight(0));
        let new = vec![
            OptionItem {
                value: 1,
                label: "Return trip".into(),
                enabled: true,
            },
            OptionItem {
                value: 0,
                label: "One-way".into(),
                enabled: true,
            },
        ];
        model.project(new.clone(), 0, true).unwrap();
        assert_eq!(model.active(), Some(0));
        model.type_text("turn", now + std::time::Duration::from_millis(100));
        assert_eq!(model.active(), Some(1));
        assert_eq!(model.selected(), Some(0));
        model.project(new.clone(), 0, false).unwrap();
        assert!(!model.is_open());
        model.project(new, 0, true).unwrap();
        assert_eq!(
            model.type_text("R", now + std::time::Duration::from_millis(200)),
            Some(1)
        );
    }
    #[test]
    fn paging_reaches_long_list_boundaries_without_committing() {
        let mut model = SelectModel::default();
        model
            .project(
                (0..256)
                    .map(|value| item(value, value == 0 || value % 3 != 0))
                    .collect(),
                0,
                true,
            )
            .unwrap();
        model.key(Key::Activate, false);
        assert_eq!(model.key(Key::PageDown(8), false), None);
        assert_eq!(model.active(), Some(11));
        model.key(Key::PageUp(8), false);
        assert_eq!(model.active(), Some(0));
        for _ in 0..64 {
            model.key(Key::PageDown(usize::MAX), false);
        }
        assert_eq!(model.active(), Some(254));
        assert_eq!(model.selected(), Some(0));
        let layout = crate::select_popup::PopupLayout::new(
            [16.0, 20.0, 200.0, 56.0],
            [320.0, 480.0],
            256,
            254,
        )
        .unwrap();
        assert!(layout.row_rect(254).is_some());
        assert_eq!(model.key(Key::Activate, false), Some(254));
    }
    #[test]
    fn value_projection_reuses_labels_and_rejects_invalid_value_atomically() {
        let mut model = SelectModel::default();
        model
            .project(vec![item(1, true), item(2, true)], 1, true)
            .unwrap();
        let options = model.options().as_ptr();
        let label = model.options()[0].label.as_ptr();
        model.key(Key::Activate, false);
        model.highlight(2);
        assert!(model.project_value(99, false).is_err());
        assert!(model.is_open());
        assert_eq!(model.selected(), Some(1));
        assert_eq!(model.active(), Some(2));
        model.project_value(2, true).unwrap();
        assert_eq!(model.options().as_ptr(), options);
        assert_eq!(model.options()[0].label.as_ptr(), label);
        model.project_value(2, false).unwrap();
        assert!(!model.is_open());
        assert_eq!(model.options().as_ptr(), options);
    }
    #[test]
    fn disabled_removed_and_keyboard_boundaries() {
        let mut model = SelectModel::default();
        model
            .project(vec![item(1, true), item(2, true)], 1, true)
            .unwrap();
        assert_eq!(model.key(Key::Up, false), None);
        assert_eq!(model.key(Key::End, false), Some(2));
        model
            .project(vec![item(1, true), item(2, true)], 2, true)
            .unwrap();
        model.key(Key::Activate, false);
        model.project(vec![item(1, true)], 1, true).unwrap();
        assert!(model.is_open());
        assert_eq!(model.active(), Some(1));
        model.key(Key::Escape, false);
        assert!(!model.is_open());
        model.project(vec![item(1, true)], 1, false).unwrap();
        model.key(Key::Activate, false);
        assert!(!model.is_open());
        model.project(vec![item(1, false)], 1, true).unwrap();
        assert_eq!(model.key(Key::Down, false), None);
        assert_eq!(model.active(), None);
    }
}
