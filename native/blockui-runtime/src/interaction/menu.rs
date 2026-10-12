//! Host-owned command-menu navigation, independent of layout and plugin code.
//! Callers provide the current visible items on every event. No application
//! callback, view, or borrowed plugin data is retained by this controller.

/// Current projection of one command. Keys belong to records, not positions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub key: u64,
    pub label: String,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movement {
    First,
    Last,
    Next,
    Previous,
}

const TYPEAHEAD_TIMEOUT_MS: u64 = 700;
const MAX_PREFIX_BYTES: usize = 256;

#[derive(Default, Debug)]
pub struct Controller {
    active: Option<u64>,
    prefix: String,
    last_input_ms: Option<u64>,
}
impl Controller {
    pub fn active(&self) -> Option<u64> {
        self.active
    }

    /// Opening starts a new keyboard session. ArrowUp can request the last item.
    pub fn open(&mut self, items: &[Item], last: bool) -> Option<u64> {
        self.clear_search();
        self.active = items
            .iter()
            .filter(|item| item.enabled)
            .map(|item| item.key)
            .nth(if last {
                items
                    .iter()
                    .filter(|item| item.enabled)
                    .count()
                    .saturating_sub(1)
            } else {
                0
            });
        self.active
    }

    pub fn close(&mut self) {
        self.active = None;
        self.clear_search();
    }

    /// Keep focus attached to its key during reordering or label updates. A
    /// removed/disabled active item falls back to the first enabled command.
    pub fn reconcile(&mut self, items: &[Item]) -> Option<u64> {
        if !items
            .iter()
            .any(|item| item.enabled && Some(item.key) == self.active)
        {
            self.active = items.iter().find(|item| item.enabled).map(|item| item.key);
        }
        self.active
    }

    /// Pointer focus shares the same active item as keyboard navigation.
    pub fn focus(&mut self, items: &[Item], key: u64) -> Option<u64> {
        if items.iter().any(|item| item.key == key && item.enabled) {
            self.active = Some(key);
            self.clear_search();
        }
        self.active
    }

    pub fn navigate(&mut self, items: &[Item], movement: Movement) -> Option<u64> {
        self.clear_search();
        let enabled = items
            .iter()
            .filter(|item| item.enabled)
            .map(|item| item.key);
        let count = enabled.clone().count();
        let current = enabled.clone().position(|key| Some(key) == self.active);
        self.active = if count == 0 {
            None
        } else {
            enabled.clone().nth(match movement {
                Movement::First => 0,
                Movement::Last => count - 1,
                Movement::Next => current.map_or(0, |i| (i + 1) % count),
                Movement::Previous => current.map_or(count - 1, |i| (i + count - 1) % count),
            })
        };
        self.active
    }

    /// Printable committed text only; the caller filters shortcuts and IME
    /// composition. Matching uses Unicode character lowercase prefix matching. Repeated
    /// single characters cycle matches; multi-character prefixes retain their
    /// current match while being extended. Timing uses the stable host clock.
    pub fn typeahead(&mut self, items: &[Item], text: &str, now_ms: u64) -> Option<u64> {
        if text.is_empty() || text.len() > MAX_PREFIX_BYTES || text.chars().any(char::is_control) {
            return self.active;
        }
        let text: String = text.chars().flat_map(char::to_lowercase).collect();
        if text.len() > MAX_PREFIX_BYTES {
            return self.active;
        }
        if self.last_input_ms.is_none_or(|last| {
            now_ms
                .checked_sub(last)
                .is_none_or(|elapsed| elapsed >= TYPEAHEAD_TIMEOUT_MS)
        }) {
            self.clear_search();
        }
        let repeated = self.prefix.chars().count() == 1 && text == self.prefix;
        let mut fresh = self.prefix.is_empty();
        if !repeated {
            if self.prefix.len() + text.len() > MAX_PREFIX_BYTES {
                self.clear_search();
                fresh = true;
            }
            self.prefix.push_str(&text);
        }
        self.last_input_ms = Some(now_ms);
        let current = items.iter().position(|item| Some(item.key) == self.active);
        let start = current.map_or(0, |i| {
            if repeated || fresh {
                (i + 1) % items.len()
            } else {
                i
            }
        });
        for offset in 0..items.len() {
            let item = &items[(start + offset) % items.len()];
            let mut candidate = item.label.trim_start().chars().flat_map(char::to_lowercase);
            if item.enabled
                && self
                    .prefix
                    .chars()
                    .all(|character| candidate.next() == Some(character))
            {
                self.active = Some(item.key);
                return self.active;
            }
        }
        self.reconcile(items)
    }

    fn clear_search(&mut self) {
        self.prefix.clear();
        self.last_input_ms = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn items() -> Vec<Item> {
        [
            (1, "Alpha", true),
            (2, "Archive", false),
            (3, "Apple", true),
            (4, "  Éditer", true),
            (5, "保存 🦀", true),
        ]
        .into_iter()
        .map(|(key, label, enabled)| Item {
            key,
            label: label.into(),
            enabled,
        })
        .collect()
    }
    #[test]
    fn arrows_wrap_and_skip_disabled_items() {
        let mut menu = Controller::default();
        let items = items();
        assert_eq!(menu.open(&items, false), Some(1));
        assert_eq!(menu.navigate(&items, Movement::Next), Some(3));
        assert_eq!(menu.navigate(&items, Movement::First), Some(1));
        assert_eq!(menu.navigate(&items, Movement::Previous), Some(5));
        assert_eq!(menu.navigate(&items, Movement::Next), Some(1));
        assert_eq!(menu.navigate(&items, Movement::Last), Some(5));
        assert_eq!(menu.open(&items, true), Some(5));
    }
    #[test]
    fn identity_survives_reorder_and_disabled_focus_is_ignored() {
        let mut menu = Controller::default();
        let mut items = items();
        assert_eq!(menu.focus(&items, 3), Some(3));
        assert_eq!(menu.focus(&items, 2), Some(3));
        items.reverse();
        assert_eq!(menu.reconcile(&items), Some(3));
        items.retain(|item| item.key != 3);
        assert_eq!(menu.reconcile(&items), Some(5));
        items[0].enabled = false;
        assert_eq!(menu.reconcile(&items), Some(4));
    }
    #[test]
    fn typeahead_cycles_and_extends_a_prefix() {
        let mut menu = Controller::default();
        let items = items();
        menu.open(&items, false);
        assert_eq!(menu.typeahead(&items, "A", 0), Some(3));
        assert_eq!(menu.typeahead(&items, "a", 10), Some(1));
        assert_eq!(menu.typeahead(&items, "p", 20), Some(3));
        assert_eq!(menu.typeahead(&items, "p", 30), Some(3));
        assert_eq!(menu.typeahead(&items, "l", 40), Some(3));
    }
    #[test]
    fn unicode_timeout_and_backwards_clock_start_new_searches() {
        let mut menu = Controller::default();
        let items = items();
        menu.open(&items, false);
        assert_eq!(menu.typeahead(&items, "É", 100), Some(4));
        assert_eq!(menu.typeahead(&items, "d", 110), Some(4));
        assert_eq!(menu.typeahead(&items, "保", 810), Some(5));
        assert_eq!(menu.typeahead(&items, "a", 0), Some(1));
    }
    #[test]
    fn committed_multicharacter_text_uses_the_same_case_mapping_as_labels() {
        let mut menu = Controller::default();
        let items = vec![Item {
            key: 1,
            label: "ΟΣ command".into(),
            enabled: true,
        }];
        assert_eq!(menu.typeahead(&items, "ΟΣ", 0), Some(1));
        let items = vec![Item {
            key: 2,
            label: "İstanbul".into(),
            enabled: true,
        }];
        menu.close();
        assert_eq!(menu.typeahead(&items, "İs", 1), Some(2));
    }
    #[test]
    fn empty_disabled_and_disappearing_items_have_no_target() {
        let mut menu = Controller::default();
        let mut items = items();
        for item in &mut items {
            item.enabled = false;
        }
        assert_eq!(menu.open(&items, true), None);
        assert_eq!(menu.navigate(&items, Movement::Previous), None);
        assert_eq!(menu.typeahead(&items, "a", 0), None);
        assert_eq!(menu.typeahead(&[], "x", 1), None);
        assert_eq!(menu.navigate(&[], Movement::Last), None);
    }
    #[test]
    fn close_and_navigation_reset_typeahead() {
        let mut menu = Controller::default();
        let items = items();
        menu.open(&items, false);
        menu.typeahead(&items, "a", 0);
        menu.navigate(&items, Movement::First);
        assert_eq!(menu.typeahead(&items, "保", 1), Some(5));
        menu.close();
        assert_eq!(menu.active(), None);
        menu.open(&items, false);
        assert_eq!(menu.typeahead(&items, "É", 2), Some(4));
    }
    #[test]
    fn search_storage_is_bounded_and_control_input_is_ignored() {
        let mut menu = Controller::default();
        let items = items();
        menu.open(&items, false);
        assert_eq!(menu.typeahead(&items, "\n", 0), Some(1));
        for tick in 0..1000 {
            menu.typeahead(&items, "保存", tick);
        }
        assert!(menu.prefix.len() <= MAX_PREFIX_BYTES);
        let active = menu.active();
        assert_eq!(menu.typeahead(&items, &"a".repeat(257), 1001), active);
    }
}
