//! Renderer-independent choice navigation, shared by native and component tests.
use crate::View;
use blockui_abi::semantics;
pub mod context_menu;
pub mod menu;
pub mod menu_hover;
pub mod menubar;
pub mod tooltip;
/// Resolve an enabled sibling for an arrow, Home or End key.
/// Horizontal tabs intentionally leave vertical arrows to their container.
pub fn choice_target(view: &View, current: u64, movement: &str) -> Option<u64> {
    let binding = view.bindings.get(&current)?;
    let node = binding.node;
    if node.disabled != 0 || !matches!(node.semantics.role, semantics::TAB | semantics::RADIO) {
        return None;
    }
    let backward = matches!(movement, "ArrowLeft" | "ArrowUp");
    let forward = matches!(movement, "ArrowRight" | "ArrowDown");
    if !backward && !forward && !matches!(movement, "Home" | "End") {
        return None;
    }
    if node.semantics.role == semantics::TAB && matches!(movement, "ArrowUp" | "ArrowDown") {
        return None;
    }
    let choices: Vec<_> = view
        .order
        .iter()
        .copied()
        .filter(|key| {
            let n = view.bindings[key].node;
            n.parent == node.parent && n.semantics.role == node.semantics.role && n.disabled == 0
        })
        .collect();
    let index = choices.iter().position(|key| *key == current)?;
    let index = match movement {
        "Home" => 0,
        "End" => choices.len() - 1,
        _ if backward => (index + choices.len() - 1) % choices.len(),
        _ => (index + 1) % choices.len(),
    };
    Some(choices[index])
}

/// Closest accordion ancestor. Nested accordions never join their parent's keys.
fn accordion_owner(view: &View, mut key: u64) -> Option<u64> {
    while let Some(binding) = view.bindings.get(&key) {
        if binding.node.semantics.role == semantics::ACCORDION {
            return Some(key);
        }
        key = binding.node.parent;
    }
    None
}
/// Vertical disclosure navigation moves focus only; activation remains explicit.
pub fn disclosure_target(view: &View, current: u64, movement: &str) -> Option<u64> {
    let node = view.bindings.get(&current)?.node;
    if node.disabled != 0
        || node.semantics.role != semantics::ACCORDION_TRIGGER
        || !matches!(movement, "ArrowUp" | "ArrowDown" | "Home" | "End")
    {
        return None;
    }
    let owner = accordion_owner(view, current)?;
    let choices: Vec<_> = view
        .order
        .iter()
        .copied()
        .filter(|key| {
            let node = view.bindings[key].node;
            node.semantics.role == semantics::ACCORDION_TRIGGER
                && node.disabled == 0
                && accordion_owner(view, *key) == Some(owner)
        })
        .collect();
    let index = choices.iter().position(|key| *key == current)?;
    Some(
        choices[match movement {
            "Home" => 0,
            "End" => choices.len() - 1,
            "ArrowUp" => (index + choices.len() - 1) % choices.len(),
            _ => (index + 1) % choices.len(),
        }],
    )
}

pub mod navigation_focus;
