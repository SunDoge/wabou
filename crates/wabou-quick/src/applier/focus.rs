//! Keyboard focusability and sequential focus navigation.

use super::*;

#[derive(Clone, Copy)]
struct FocusCandidate {
    solid_id: u32,
    tab_index: i32,
    document_order: usize,
}

impl Applier {
    pub(super) fn rebuild_focus_order(&mut self, placed: &[PlacedNode]) {
        let atoms = self.atoms.borrow();
        let attribute = |declared: &Declared, wanted: &str| {
            declared.attrs.iter().find_map(|(name, value)| {
                (atoms.resolve(*name) == Some(wanted)).then(|| value.to_string())
            })
        };
        let modal_node = placed.iter().rev().find_map(|placed| {
            let declared = self.node_store.declared.get(&placed.node_id)?;
            (placed.paint.overlay_plane == OverlayPlane::Modal
                && attribute(declared, "aria-modal").as_deref() == Some("true"))
            .then_some(placed.node_id)
        });
        let mut candidates = Vec::new();
        let mut focusable_targets = HashSet::new();
        for (document_order, placed) in placed.iter().enumerate() {
            if placed.node_id == self.node_store.root
                || modal_node.is_some_and(|modal| {
                    !self.node_store.is_logical_descendant(placed.node_id, modal)
                })
            {
                continue;
            }
            let Some(declared) = self.node_store.declared.get(&placed.node_id) else {
                continue;
            };
            if attribute(declared, "disabled").is_some()
                || attribute(declared, "aria-disabled").as_deref() == Some("true")
                || attribute(declared, "aria-hidden").as_deref() == Some("true")
            {
                continue;
            }
            let explicit_tab_index = attribute(declared, "tabIndex")
                .or_else(|| attribute(declared, "tabindex"))
                .and_then(|value| value.parse::<i32>().ok());
            let tag = declared
                .tag
                .and_then(|tag| atoms.resolve(tag))
                .unwrap_or("");
            let role = attribute(declared, "role").unwrap_or_default();
            let widget_focusable = self
                .widget_manager
                .widgets
                .get(&placed.node_id)
                .is_some_and(|widget| widget.accepts_focus());
            let intrinsic_focusable = widget_focusable
                || matches!(tag, "button" | "input" | "textarea" | "select")
                || (tag == "a" && attribute(declared, "href").is_some())
                || matches!(
                    role.as_str(),
                    "button"
                        | "checkbox"
                        | "combobox"
                        | "link"
                        | "listbox"
                        | "radio"
                        | "switch"
                        | "textbox"
                );
            if explicit_tab_index.is_none() && !intrinsic_focusable {
                continue;
            }
            let Some(solid_id) = self.node_store.solid_id_for_node(placed.node_id) else {
                continue;
            };
            focusable_targets.insert(solid_id);
            let tab_index = explicit_tab_index.unwrap_or(0);
            if tab_index >= 0 {
                candidates.push(FocusCandidate {
                    solid_id,
                    tab_index,
                    document_order,
                });
            }
        }
        drop(atoms);
        candidates.sort_by_key(|candidate| {
            (
                candidate.tab_index == 0,
                candidate.tab_index.max(0),
                candidate.document_order,
            )
        });
        self.input.focus_order = candidates
            .into_iter()
            .map(|candidate| candidate.solid_id)
            .collect();
        self.input.focusable_targets = focusable_targets;

        let focused_is_valid = self
            .input
            .focused_target
            .is_none_or(|focused| self.input.focusable_targets.contains(&focused));
        if !focused_is_valid {
            let fallback = modal_node.and_then(|_| self.input.focus_order.first().copied());
            self.set_focused_target(fallback);
        }
    }

    pub(super) fn pointer_focus_target(&self, target: Option<u32>) -> Option<u32> {
        let mut node =
            target.and_then(|target| self.node_store.solid_to_node.get(&target).copied());
        while let Some(current) = node {
            if let Some(solid_id) = self.node_store.solid_id_for_node(current)
                && self.input.focusable_targets.contains(&solid_id)
            {
                return Some(solid_id);
            }
            node = self.node_store.logical_parent.get(&current).copied();
        }
        None
    }

    pub(super) fn advance_focus(&mut self, reverse: bool) -> Option<u32> {
        if self.input.focus_order.is_empty() {
            return None;
        }
        let current = self.input.focused_target.and_then(|focused| {
            self.input
                .focus_order
                .iter()
                .position(|target| *target == focused)
        });
        let index = match (current, reverse) {
            (Some(0), true) | (None, true) => self.input.focus_order.len() - 1,
            (Some(index), true) => index - 1,
            (Some(index), false) => (index + 1) % self.input.focus_order.len(),
            (None, false) => 0,
        };
        let target = self.input.focus_order[index];
        self.set_focused_target(Some(target));
        Some(target)
    }
}
