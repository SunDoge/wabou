//! Top-level menubar decisions shared by renderer hosts. The caller supplies the
//! current projection and dispatches decisions through its current actor table.
//! No callbacks, timers, allocations or plugin data are retained here.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    pub key: u64,
    pub enabled: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Previous,
    Next,
    First,
    Last,
    OpenFirst,
    OpenLast,
    Hover(u64),
    Escape,
    Tab,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Decision {
    pub focus: Option<u64>,
    pub close: Option<u64>,
    pub open: Option<u64>,
    /// Opening by ArrowUp requests the last enabled child command.
    pub last_child: bool,
}

/// Top-level arrows preserve an open menu session; hover switches menus only
/// during that session. Tab closes the session and lets the host move focus.
/// `current` is the root trigger owning focus, even when focus is in its child.
pub fn resolve(items: &[Item], current: u64, open: Option<u64>, input: Input) -> Decision {
    let enabled = || items.iter().filter(|item| item.enabled && item.key != 0);
    let open = open.filter(|key| enabled().any(|item| item.key == *key));
    if matches!(input, Input::Escape | Input::Tab) {
        return Decision {
            focus: if input == Input::Escape { open } else { None },
            close: open,
            ..Decision::default()
        };
    }
    if let Input::Hover(key) = input {
        if open.is_none() || open == Some(key) || !enabled().any(|item| item.key == key) {
            return Decision::default();
        }
        return Decision {
            focus: Some(key),
            close: open,
            open: Some(key),
            last_child: false,
        };
    }
    let count = enabled().count();
    if count == 0 {
        return Decision::default();
    }
    let position = enabled().position(|item| item.key == current);
    if matches!(input, Input::OpenFirst | Input::OpenLast) {
        if position.is_none() {
            return Decision::default();
        }
        return Decision {
            focus: Some(current),
            close: open.filter(|key| *key != current),
            open: Some(current),
            last_child: input == Input::OpenLast,
        };
    }
    let index = match input {
        Input::Previous => position.map_or(count - 1, |i| (i + count - 1) % count),
        Input::Next => position.map_or(0, |i| (i + 1) % count),
        Input::First => 0,
        Input::Last => count - 1,
        _ => return Decision::default(),
    };
    let target = enabled().nth(index).unwrap().key;
    Decision {
        focus: Some(target),
        close: open.filter(|key| *key != target),
        open: open.map(|_| target),
        last_child: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ITEMS: [Item; 3] = [
        Item {
            key: 1,
            enabled: true,
        },
        Item {
            key: 2,
            enabled: false,
        },
        Item {
            key: 3,
            enabled: true,
        },
    ];
    #[test]
    fn closed_arrows_wrap_skip_disabled_and_only_move_focus() {
        assert_eq!(
            resolve(&ITEMS, 1, None, Input::Next),
            Decision {
                focus: Some(3),
                ..Decision::default()
            }
        );
        assert_eq!(resolve(&ITEMS, 1, None, Input::Previous).focus, Some(3));
        assert_eq!(resolve(&ITEMS, 3, None, Input::Next).focus, Some(1));
        assert_eq!(resolve(&ITEMS, 3, None, Input::First).focus, Some(1));
        assert_eq!(resolve(&ITEMS, 1, None, Input::Last).focus, Some(3));
    }
    #[test]
    fn open_arrows_switch_one_root_menu() {
        assert_eq!(
            resolve(&ITEMS, 1, Some(1), Input::Next),
            Decision {
                focus: Some(3),
                close: Some(1),
                open: Some(3),
                last_child: false
            }
        );
    }
    #[test]
    fn hover_requires_session_and_enabled_distinct_target() {
        assert_eq!(
            resolve(&ITEMS, 1, None, Input::Hover(3)),
            Decision::default()
        );
        for target in [0, 1, 2, 9] {
            assert_eq!(
                resolve(&ITEMS, 1, Some(1), Input::Hover(target)),
                Decision::default()
            );
        }
        assert_eq!(resolve(&ITEMS, 1, Some(1), Input::Hover(3)).open, Some(3));
    }
    #[test]
    fn vertical_arrows_request_first_or_last_child() {
        assert!(!resolve(&ITEMS, 1, None, Input::OpenFirst).last_child);
        let result = resolve(&ITEMS, 3, Some(1), Input::OpenLast);
        assert_eq!(
            result,
            Decision {
                focus: Some(3),
                close: Some(1),
                open: Some(3),
                last_child: true
            }
        );
        assert_eq!(
            resolve(&ITEMS, 2, None, Input::OpenFirst),
            Decision::default()
        );
    }
    #[test]
    fn escape_restores_root_and_tab_leaves_focus_to_host() {
        assert_eq!(
            resolve(&ITEMS, 1, Some(1), Input::Escape),
            Decision {
                focus: Some(1),
                close: Some(1),
                ..Decision::default()
            }
        );
        assert_eq!(
            resolve(&ITEMS, 1, Some(1), Input::Tab),
            Decision {
                close: Some(1),
                ..Decision::default()
            }
        );
    }
    #[test]
    fn current_projection_handles_reorder_removal_and_empty() {
        let reversed = [ITEMS[2], ITEMS[0]];
        assert_eq!(resolve(&reversed, 3, None, Input::Next).focus, Some(1));
        assert_eq!(
            resolve(&ITEMS, 99, Some(99), Input::Next),
            Decision {
                focus: Some(1),
                ..Decision::default()
            }
        );
        assert_eq!(resolve(&[], 1, Some(1), Input::Next), Decision::default());
        assert_eq!(
            resolve(&[ITEMS[1]], 2, Some(2), Input::Last),
            Decision::default()
        );
    }
}
