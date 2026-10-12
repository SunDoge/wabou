//! Dependency-free accessible presentation. Identities are actor-local node keys.
use crate::*;
pub const DEFAULT: u32 = 0;
pub const HEADING: u32 = 1;
pub const NAVIGATION: u32 = 2;
pub const TAB_LIST: u32 = 3;
pub const TAB: u32 = 4;
pub const TAB_PANEL: u32 = 5;
pub const RADIO_GROUP: u32 = 6;
pub const RADIO: u32 = 7;
pub const SWITCH: u32 = 8;
pub const ALERT: u32 = 9;
pub const STATUS: u32 = 10;
pub const ACCORDION: u32 = 11;
pub const DISCLOSURE_PANEL: u32 = 12;
pub const ACCORDION_TRIGGER: u32 = 13;
pub const ACCORDION_ITEM: u32 = 14;
/// Nonmodal host viewport, independently laid out above application content.
pub const TOAST_VIEWPORT: u32 = 15;
/// Anchored nonmodal scrollport. `labelled_by` identifies its trigger/anchor.
/// The node action dismisses the panel; hidden panels have no children.
pub const POPOVER_PANEL: u32 = 16;
/// A button controlling a matching popover, with explicit expanded state.
pub const POPOVER_TRIGGER: u32 = 17;
pub const HORIZONTAL_SEPARATOR: u32 = 18;
pub const VERTICAL_SEPARATOR: u32 = 19;
/// Noninteractive short text label; button_variant selects its appearance.
pub const BADGE: u32 = 20;
/// Host-controlled noninteractive floating text description.
pub const TOOLTIP: u32 = 21;
pub const TABLE: u32 = 22;
pub const TABLE_ROW: u32 = 23;
pub const COLUMN_HEADER: u32 = 24;
pub const CELL: u32 = 25;
pub const MENU_PANEL: u32 = 26;
pub const MENU_TRIGGER: u32 = 27;
pub const MENU_ITEM: u32 = 28;
pub const MENU_CHECKBOX: u32 = 29;
pub const MENU_RADIO: u32 = 30;
pub const MENU_GROUP: u32 = 31;
pub const MENU_LABEL: u32 = 32;
pub const SUBMENU: u32 = 33;
pub const SUBMENU_TRIGGER: u32 = 34;
/// Horizontal composite of root command-menu containers.
pub const MENUBAR: u32 = 35;
/// Explicit action target accepting context activation from the stable host.
pub const CONTEXT_TRIGGER: u32 = 36;
/// Focusable content region accepting context activation without primary click.
pub const CONTEXT_SURFACE: u32 = 37;
pub const fn is_context_trigger(role: u32) -> bool {
    matches!(role, CONTEXT_TRIGGER | CONTEXT_SURFACE)
}
pub const fn is_menu_item(role: u32) -> bool {
    matches!(
        role,
        MENU_ITEM | MENU_CHECKBOX | MENU_RADIO | SUBMENU_TRIGGER
    )
}
/// Nonmodal anchored panels share placement and dismissal rules.
pub const fn is_anchored_panel(role: u32) -> bool {
    matches!(role, POPOVER_PANEL | MENU_PANEL)
}
pub const fn is_anchored_trigger(role: u32) -> bool {
    matches!(
        role,
        POPOVER_TRIGGER | MENU_TRIGGER | SUBMENU_TRIGGER | CONTEXT_TRIGGER | CONTEXT_SURFACE
    )
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Semantics {
    pub role: u32,
    /// Column sort: 0 absent, 1 ascending, 2 descending, 3 other.
    pub sort_direction: u32,
    /// Selected/checked choice (0/1). Switch state comes from the checkbox recipe.
    pub selected: u32,
    /// Current page: 0 absent, 1 page. Ordinary buttons only.
    pub current: u32,
    /// Composite-widget Tab entry; ordinary controls use their default behavior.
    pub tab_stop: u32,
    /// Inactive content-panel placeholder: no children, layout or accessibility content.
    pub hidden: u32,
    /// Heading level 1..=6; other roles require zero.
    pub level: u32,
    /// Disclosure state: 0 absent, 1 collapsed, 2 expanded.
    pub expanded: u32,
    /// Expanded accordion header cannot collapse itself but stays keyboard-focusable.
    pub locked: u32,
    pub controls: u64,
    pub labelled_by: u64,
    pub described_by: u64,
}
impl Semantics {
    pub fn validate(self, kind: u32) -> Result<(), &'static str> {
        let compatible = match self.role {
            DEFAULT => true,
            TABLE | COLUMN_HEADER | CELL => kind == COLUMN,
            TABLE_ROW | MENUBAR => kind == ROW,
            TOAST_VIEWPORT => kind == SCROLL,
            POPOVER_PANEL | MENU_PANEL => kind == SCROLL && self.labelled_by != 0,
            POPOVER_TRIGGER | MENU_TRIGGER | SUBMENU_TRIGGER | CONTEXT_TRIGGER => {
                kind == BUTTON && self.expanded != 0 && self.controls != 0
            }
            CONTEXT_SURFACE => {
                kind == COLUMN && self.expanded != 0 && self.controls != 0 && self.labelled_by != 0
            }
            HORIZONTAL_SEPARATOR | VERTICAL_SEPARATOR => kind == COLUMN,
            BADGE | TOOLTIP | MENU_LABEL => kind == LABEL,
            MENU_GROUP => kind == COLUMN && self.labelled_by != 0,
            HEADING => matches!(kind, LABEL | COLUMN) && (1..=6).contains(&self.level),
            NAVIGATION | TAB_LIST | RADIO_GROUP | ALERT | STATUS => matches!(kind, COLUMN | ROW),
            TAB | RADIO | MENU_ITEM | MENU_CHECKBOX | MENU_RADIO => kind == BUTTON,
            SUBMENU | TAB_PANEL | ACCORDION | ACCORDION_ITEM | DISCLOSURE_PANEL => kind == COLUMN,
            ACCORDION_TRIGGER => kind == BUTTON && self.expanded != 0 && self.controls != 0,
            SWITCH => kind == CHECKBOX,
            _ => false,
        };
        if (self.described_by != 0
            && !matches!(kind, BUTTON | ENTRY | TEXTAREA | CHECKBOX | SLIDER | SELECT))
            || !compatible
            || self.sort_direction > 3
            || (self.sort_direction != 0 && self.role != COLUMN_HEADER)
            || (self.role != HEADING && self.level != 0)
            || self.current > 1
            || (self.current != 0 && (self.role != DEFAULT || kind != BUTTON))
            || self.selected > 1
            || (self.selected != 0
                && !matches!(self.role, TAB | RADIO | MENU_CHECKBOX | MENU_RADIO))
            || self.tab_stop > 1
            || (self.tab_stop != 0 && !matches!(self.role, TAB | RADIO | TAB_PANEL))
            || self.hidden > 1
            || (self.hidden != 0
                && !(matches!(
                    self.role,
                    TAB_PANEL | DISCLOSURE_PANEL | POPOVER_PANEL | MENU_PANEL
                ) || (self.role == DEFAULT && matches!(kind, crate::COLUMN | crate::ROW))))
            || (self.hidden != 0 && self.tab_stop != 0)
            || self.locked > 1
            || (self.locked != 0 && (self.role != ACCORDION_TRIGGER || self.expanded != 2))
            || self.expanded > 2
            || (self.expanded != 0
                && ((kind != BUTTON && self.role != CONTEXT_SURFACE)
                    || !matches!(
                        self.role,
                        DEFAULT
                            | ACCORDION_TRIGGER
                            | POPOVER_TRIGGER
                            | MENU_TRIGGER
                            | SUBMENU_TRIGGER
                            | CONTEXT_TRIGGER
                            | CONTEXT_SURFACE
                    )))
            || (self.controls != 0 && self.role != TAB && self.expanded == 0)
            || (self.labelled_by != 0
                && !(self.role == DEFAULT && kind == CHECKBOX)
                && !matches!(
                    self.role,
                    MENU_GROUP
                        | CONTEXT_SURFACE
                        | TABLE
                        | TAB_PANEL
                        | TAB_LIST
                        | RADIO_GROUP
                        | ALERT
                        | STATUS
                        | DISCLOSURE_PANEL
                        | POPOVER_PANEL
                        | MENU_PANEL
                ))
        {
            return Err("invalid widget semantics");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn menu_groups_require_column_and_title_link() {
        let group = Semantics {
            role: MENU_GROUP,
            labelled_by: 42,
            ..Default::default()
        };
        assert!(group.validate(COLUMN).is_ok());
        assert!(group.validate(BUTTON).is_err());
        assert!(
            Semantics {
                labelled_by: 0,
                ..group
            }
            .validate(COLUMN)
            .is_err()
        );
        assert!(
            Semantics {
                role: MENU_LABEL,
                ..Default::default()
            }
            .validate(LABEL)
            .is_ok()
        );
        assert!(
            Semantics {
                role: MENU_LABEL,
                ..Default::default()
            }
            .validate(BUTTON)
            .is_err()
        );
    }
    #[test]
    fn checked_menu_items_require_button_and_binary_state() {
        let checked = Semantics {
            role: MENU_CHECKBOX,
            selected: 1,
            ..Default::default()
        };
        assert!(checked.validate(BUTTON).is_ok());
        assert!(
            Semantics {
                role: MENU_RADIO,
                ..checked
            }
            .validate(BUTTON)
            .is_ok()
        );
        assert!(
            Semantics {
                role: MENU_RADIO,
                ..checked
            }
            .validate(CHECKBOX)
            .is_err()
        );
        assert!(checked.validate(CHECKBOX).is_err());
        assert!(
            Semantics {
                selected: 2,
                ..checked
            }
            .validate(BUTTON)
            .is_err()
        );
        assert!(
            Semantics {
                role: MENU_ITEM,
                ..checked
            }
            .validate(BUTTON)
            .is_err()
        );
    }
    #[test]
    fn sorting_is_only_valid_on_column_headers() {
        for direction in 1..=3 {
            assert!(
                Semantics {
                    role: COLUMN_HEADER,
                    sort_direction: direction,
                    ..Default::default()
                }
                .validate(COLUMN)
                .is_ok()
            );
            assert!(
                Semantics {
                    role: CELL,
                    sort_direction: direction,
                    ..Default::default()
                }
                .validate(COLUMN)
                .is_err()
            );
        }
        assert!(
            Semantics {
                role: COLUMN_HEADER,
                sort_direction: 4,
                ..Default::default()
            }
            .validate(COLUMN)
            .is_err()
        );
    }
    #[test]
    fn compact_checkbox_accepts_an_external_name() {
        let named = Semantics {
            labelled_by: 42,
            ..Default::default()
        };
        assert!(named.validate(CHECKBOX).is_ok());
        for kind in [LABEL, BUTTON, ENTRY, ROW, COLUMN] {
            assert!(named.validate(kind).is_err());
        }
    }
    #[test]
    fn labels_and_dividers_are_noninteractive_semantics() {
        for (role, kind) in [
            (BADGE, LABEL),
            (HORIZONTAL_SEPARATOR, COLUMN),
            (VERTICAL_SEPARATOR, COLUMN),
        ] {
            let semantics = Semantics {
                role,
                ..Default::default()
            };
            assert!(semantics.validate(kind).is_ok());
            assert!(semantics.validate(BUTTON).is_err());
            for invalid in [
                Semantics {
                    tab_stop: 1,
                    ..semantics
                },
                Semantics {
                    selected: 1,
                    ..semantics
                },
                Semantics {
                    expanded: 1,
                    ..semantics
                },
                Semantics {
                    hidden: 1,
                    ..semantics
                },
            ] {
                assert!(invalid.validate(kind).is_err());
            }
        }
    }
    #[test]
    fn popover_protocol_requires_explicit_trigger_and_anchor() {
        let trigger = Semantics {
            role: POPOVER_TRIGGER,
            expanded: 2,
            controls: 3,
            ..Default::default()
        };
        assert!(trigger.validate(BUTTON).is_ok());
        assert!(trigger.validate(SCROLL).is_err());
        for invalid in [
            Semantics {
                expanded: 0,
                ..trigger
            },
            Semantics {
                controls: 0,
                ..trigger
            },
            Semantics {
                locked: 1,
                ..trigger
            },
            Semantics {
                hidden: 1,
                ..trigger
            },
            Semantics {
                tab_stop: 1,
                ..trigger
            },
        ] {
            assert!(invalid.validate(BUTTON).is_err());
        }
        let panel = Semantics {
            role: POPOVER_PANEL,
            labelled_by: 2,
            ..Default::default()
        };
        assert!(panel.validate(SCROLL).is_ok());
        assert!(panel.validate(COLUMN).is_err());
        assert!(Semantics { hidden: 1, ..panel }.validate(SCROLL).is_ok());
        for invalid in [
            Semantics {
                labelled_by: 0,
                ..panel
            },
            Semantics {
                expanded: 2,
                ..panel
            },
            Semantics {
                controls: 3,
                ..panel
            },
            Semantics {
                selected: 1,
                ..panel
            },
            Semantics {
                tab_stop: 1,
                ..panel
            },
        ] {
            assert!(invalid.validate(SCROLL).is_err());
        }
    }
    #[test]
    fn roles_and_states_must_match_the_control_kind() {
        assert!(
            Semantics {
                role: SWITCH,
                ..Default::default()
            }
            .validate(CHECKBOX)
            .is_ok()
        );
        assert!(
            Semantics {
                role: SWITCH,
                ..Default::default()
            }
            .validate(BUTTON)
            .is_err()
        );
        for invalid in [
            Semantics {
                role: 999,
                ..Default::default()
            },
            Semantics {
                role: HEADING,
                level: 7,
                ..Default::default()
            },
            Semantics {
                selected: 1,
                ..Default::default()
            },
            Semantics {
                role: TAB_PANEL,
                hidden: 1,
                tab_stop: 1,
                ..Default::default()
            },
            Semantics {
                expanded: 3,
                ..Default::default()
            },
            Semantics {
                controls: 12,
                ..Default::default()
            },
        ] {
            assert!(invalid.validate(LABEL).is_err());
        }
        assert!(
            Semantics {
                role: HEADING,
                level: 1,
                ..Default::default()
            }
            .validate(LABEL)
            .is_ok()
        );
        assert!(
            Semantics {
                expanded: 2,
                controls: 12,
                ..Default::default()
            }
            .validate(BUTTON)
            .is_ok()
        );
    }
    #[test]
    fn disclosure_and_heading_container_fields_are_restricted_to_their_roles() {
        let panel = Semantics {
            role: DISCLOSURE_PANEL,
            hidden: 1,
            labelled_by: 2,
            ..Default::default()
        };
        assert!(panel.validate(COLUMN).is_ok());
        assert!(panel.validate(BUTTON).is_err());
        assert!(
            Semantics {
                tab_stop: 1,
                ..panel
            }
            .validate(COLUMN)
            .is_err()
        );
        let trigger = Semantics {
            role: ACCORDION_TRIGGER,
            expanded: 2,
            controls: 3,
            ..Default::default()
        };
        assert!(trigger.validate(BUTTON).is_ok());
        assert!(
            Semantics {
                locked: 1,
                ..trigger
            }
            .validate(BUTTON)
            .is_ok()
        );
        assert!(
            Semantics {
                locked: 1,
                expanded: 1,
                ..trigger
            }
            .validate(BUTTON)
            .is_err()
        );
        assert!(
            Semantics {
                controls: 0,
                ..trigger
            }
            .validate(BUTTON)
            .is_err()
        );
        assert!(
            Semantics {
                expanded: 0,
                ..trigger
            }
            .validate(BUTTON)
            .is_err()
        );
        assert!(
            Semantics {
                role: HEADING,
                level: 3,
                ..Default::default()
            }
            .validate(COLUMN)
            .is_ok()
        );
        assert!(
            Semantics {
                role: HEADING,
                level: 0,
                ..Default::default()
            }
            .validate(COLUMN)
            .is_err()
        );
    }
}

#[cfg(test)]
mod current_page_tests {
    use super::*;
    #[test]
    fn current_page_is_explicit_and_rejects_incompatible_recipes() {
        let current = Semantics {
            current: 1,
            ..Default::default()
        };
        assert!(current.validate(BUTTON).is_ok());
        for kind in [TEXT, COLUMN, ROW, CHECKBOX, ENTRY] {
            assert!(current.validate(kind).is_err());
        }
        assert!(
            Semantics {
                current: 2,
                ..Default::default()
            }
            .validate(BUTTON)
            .is_err()
        );
        assert!(
            Semantics {
                role: MENU_ITEM,
                ..current
            }
            .validate(BUTTON)
            .is_err()
        );
        assert!(Semantics::default().validate(BUTTON).is_ok());
    }
}
