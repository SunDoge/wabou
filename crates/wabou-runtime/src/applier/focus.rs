//! Keyboard focusability and sequential focus navigation.

use super::*;

#[derive(Clone, Copy)]
struct FocusCandidate {
    solid_id: u32,
    tab_index: i32,
    document_order: usize,
}

impl Applier {
    pub(super) fn handle_focused_input(&mut self, input: UiEvent) -> EventResponse {
        if matches!(&input, UiEvent::Key(key) if key.phase == KeyPhase::Down)
            || matches!(
                &input,
                UiEvent::TextInput(_) | UiEvent::Ime(_) | UiEvent::Paste(_)
            )
        {
            self.text_selection.last_click = None;
        }

        let keydown_dispatched = match self.dispatch_focused_keydown(&input) {
            Ok(dispatched) => dispatched,
            Err(response) => return response,
        };

        let widget_response = self
            .input
            .focused_target
            .and_then(|target| self.handle_widget_event(target, &input));
        if widget_response.is_none()
            && let Some(response) = self.handle_unclaimed_focused_input(&input)
        {
            return response;
        }

        let handled =
            self.dispatch_focused_dom_input(input, keydown_dispatched, widget_response.is_some());
        widget_response.map_or_else(
            || Self::response(handled),
            |widget| EventResponse {
                handled: widget.handled || handled,
                request_redraw: widget.request_redraw || handled,
                consume_key_text: widget.consume_key_text,
                text_input: widget.text_input,
                clipboard: widget.clipboard,
            },
        )
    }

    fn dispatch_focused_keydown(&mut self, input: &UiEvent) -> Result<bool, EventResponse> {
        let UiEvent::Key(key) = input else {
            return Ok(false);
        };
        if key.phase != KeyPhase::Down {
            return Ok(false);
        }
        let Some(target) = self.input.focused_target else {
            return Ok(false);
        };

        // Application shortcuts get first refusal before a focused Rust
        // widget consumes the key.
        let payload = key_event_payload(key);
        let (dispatched, prevented) =
            self.dispatch_cancellable_json(target, event::KEYDOWN, payload);
        if prevented {
            Err(EventResponse {
                handled: true,
                request_redraw: true,
                consume_key_text: true,
                text_input: None,
                clipboard: None,
            })
        } else {
            Ok(dispatched)
        }
    }

    fn handle_unclaimed_focused_input(&mut self, input: &UiEvent) -> Option<EventResponse> {
        let UiEvent::Key(key) = input else {
            return None;
        };
        if key.matches_standard_shortcut(wabou_shell::StandardShortcut::Copy)
            && let Some(text) = self.selected_text()
        {
            return Some(EventResponse {
                handled: true,
                request_redraw: false,
                consume_key_text: false,
                text_input: None,
                clipboard: Some(wabou_shell::ClipboardRequest::Write(text)),
            });
        }
        if key.matches_standard_shortcut(wabou_shell::StandardShortcut::SelectAll)
            && self.select_all_text()
        {
            self.sync_text_selection_change();
            return Some(EventResponse {
                handled: true,
                request_redraw: true,
                consume_key_text: false,
                text_input: None,
                clipboard: None,
            });
        }
        if key.phase != KeyPhase::Down
            || key.key != "Tab"
            || key.modifiers.control()
            || key.modifiers.alt()
            || key.modifiers.meta()
        {
            return None;
        }
        let target = self.advance_focus(key.modifiers.shift())?;
        Some(EventResponse {
            handled: true,
            request_redraw: true,
            consume_key_text: true,
            text_input: Some(self.is_text_input_target(target)),
            clipboard: None,
        })
    }

    fn dispatch_focused_dom_input(
        &mut self,
        input: UiEvent,
        keydown_dispatched: bool,
        widget_handled: bool,
    ) -> bool {
        match input {
            UiEvent::Key(key) if key.phase == KeyPhase::Down => keydown_dispatched,
            UiEvent::Key(key) if key.phase == KeyPhase::Up => {
                self.input.focused_target.is_some_and(|target| {
                    let payload = key_event_payload(&key);
                    self.dispatch_json(target, event::KEYUP, &payload)
                })
            }
            UiEvent::TextInput(text)
            | UiEvent::Ime(wabou_shell::ImeEvent::Commit(text))
            | UiEvent::Paste(text) => self.input.focused_target.is_some_and(|target| {
                let payload = serde_json::json!({ "data": text }).to_string();
                self.dispatch_json(target, event::IMECOMMIT, &payload)
            }),
            UiEvent::Ime(_) => widget_handled,
            _ => unreachable!("focused input routing received a non-input event"),
        }
    }

    pub(super) fn handle_window_focus(&mut self, focused: bool) -> EventResponse {
        let mut changed = if focused {
            false
        } else {
            self.text_selection.last_click = None;
            self.cancel_active_pointer_gesture()
        };
        changed |= self.set_window_focused(focused);
        EventResponse {
            handled: changed,
            request_redraw: changed,
            consume_key_text: false,
            text_input: Some(
                focused
                    && self
                        .input
                        .focused_target
                        .is_some_and(|target| self.is_text_input_target(target)),
            ),
            clipboard: None,
        }
    }

    pub(super) fn rebuild_focus_order(&mut self, placed: &[PlacedNode]) {
        let atoms = self.atoms.borrow();
        let attribute = |declared: &Declared, wanted: &str| {
            declared.attrs.iter().find_map(|(name, value)| {
                (atoms.resolve(*name) == Some(wanted)).then(|| value.to_string())
            })
        };
        let modal = placed.iter().enumerate().rev().find_map(|(index, placed)| {
            let declared = self.node_store.declared.get(&placed.node_id)?;
            (placed.paint.overlay_plane == OverlayPlane::Modal
                && attribute(declared, "focusScope").as_deref() == Some("contain"))
            .then_some((index, placed.node_id))
        });
        // A portal opened from inside a modal is a physical sibling under the
        // shared modal plane, not a logical descendant of the dialog. Treat
        // later modal-plane roots as part of the active modal scope so their
        // controls remain focusable without admitting older/background modals.
        let supplemental_modal_roots = modal.map_or_else(Vec::new, |(index, _)| {
            placed[index + 1..]
                .iter()
                .filter(|node| node.paint.overlay_plane == OverlayPlane::Modal)
                .map(|node| node.node_id)
                .collect::<Vec<_>>()
        });
        let inside_active_modal = |node| {
            modal.is_none_or(|(_, modal_node)| {
                self.node_store.is_logical_descendant(node, modal_node)
                    || supplemental_modal_roots
                        .iter()
                        .any(|root| self.node_store.is_logical_descendant(node, *root))
            })
        };
        let mut candidates = Vec::new();
        let mut focusable_targets = HashSet::new();
        for (document_order, placed) in placed.iter().enumerate() {
            if placed.node_id == self.node_store.root
                || !inside_active_modal(placed.node_id)
                || subtree_has_attribute(&self.node_store, &atoms, placed.node_id, "inert", None)
            {
                continue;
            }
            let Some(declared) = self.node_store.declared.get(&placed.node_id) else {
                continue;
            };
            // `tabIndex` is the normalized Wabou focus contract. Whether a
            // disabled component participates is decided by its JS primitive,
            // rather than inferred here from browser attribute conventions.
            let explicit_tab_index =
                attribute(declared, "tabIndex").and_then(|value| value.parse::<i32>().ok());
            if explicit_tab_index.is_none() {
                continue;
            }
            let Some(solid_id) = self.node_store.solid_id_for_node(placed.node_id) else {
                continue;
            };
            focusable_targets.insert(solid_id);
            let tab_index = explicit_tab_index.expect("checked above");
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
            let fallback = modal.and_then(|_| self.input.focus_order.first().copied());
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
