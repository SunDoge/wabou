//! Recover a disabled button's focus within its current navigation region.
//! Operates only on the current host-owned projection, with no retained callbacks.
use crate::View;
use blockui_abi::{BUTTON, semantics};

fn owner(view: &View, mut key: u64) -> Option<u64> {
    while let Some(binding) = view.bindings.get(&key) {
        if binding.node.semantics.hidden != 0 {
            return None;
        }
        if binding.node.semantics.role == semantics::NAVIGATION {
            return Some(key);
        }
        key = binding.node.parent;
    }
    None
}

pub fn recover(view: &View, focused: u64) -> Option<u64> {
    let binding = view.bindings.get(&focused)?;
    if binding.node.kind != BUTTON
        || binding.node.disabled == 0
        || binding.node.semantics.role != semantics::DEFAULT
    {
        return None;
    }
    let navigation = owner(view, focused)?;
    let position = view.order.iter().position(|key| *key == focused)?;
    view.order
        .iter()
        .enumerate()
        .filter(|(_, key)| {
            let node = &view.bindings[key].node;
            node.kind == BUTTON
                && node.disabled == 0
                && node.semantics.role == semantics::DEFAULT
                && owner(view, **key) == Some(navigation)
        })
        .min_by_key(|(index, key)| {
            (
                view.bindings[key].node.semantics.current != 1,
                index.abs_diff(position),
                *index > position,
            )
        })
        .map(|(_, key)| *key)
}
