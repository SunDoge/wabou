//! Application-level identities for GPUI windows.

use slotmap::{Key as _, KeyData, SlotMap};

slotmap::new_key_type! {
    struct GpuiWindowSlot;
}

/// Shared registry separating stable Wabou window identities from GPUI's
/// process-local handles.
#[derive(Clone, Default)]
pub(crate) struct GpuiWindowRegistry {
    entries: std::rc::Rc<
        std::cell::RefCell<SlotMap<GpuiWindowSlot, Option<gpui_shell::gpui::AnyWindowHandle>>>,
    >,
}

impl GpuiWindowRegistry {
    pub(crate) fn reserve(&self) -> gpui_shell::WindowResourceKey {
        let key = self.entries.borrow_mut().insert(None);
        gpui_shell::WindowResourceKey::from_ffi(key.data().as_ffi())
            .expect("SlotMap generated an invalid window resource key")
    }

    pub(crate) fn attach(
        &self,
        key: gpui_shell::WindowResourceKey,
        handle: gpui_shell::gpui::AnyWindowHandle,
    ) -> bool {
        let key = GpuiWindowSlot::from(KeyData::from_ffi(key.as_ffi()));
        let mut entries = self.entries.borrow_mut();
        let Some(entry) = entries.get_mut(key) else {
            return false;
        };
        *entry = Some(handle);
        true
    }

    pub(crate) fn resolve(
        &self,
        key: gpui_shell::WindowResourceKey,
    ) -> Option<gpui_shell::gpui::AnyWindowHandle> {
        let key = GpuiWindowSlot::from(KeyData::from_ffi(key.as_ffi()));
        self.entries.borrow().get(key).copied().flatten()
    }

    pub(crate) fn remove(
        &self,
        key: gpui_shell::WindowResourceKey,
    ) -> Option<gpui_shell::gpui::AnyWindowHandle> {
        let key = GpuiWindowSlot::from(KeyData::from_ffi(key.as_ffi()));
        self.entries.borrow_mut().remove(key).flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservations_match_initial_keys_and_reject_removed_generations() {
        let registry = GpuiWindowRegistry::default();
        let first = registry.reserve();
        let second = registry.reserve();
        assert_eq!(first, gpui_shell::initial_window_resource_key(0));
        assert_eq!(second, gpui_shell::initial_window_resource_key(1));
        assert_eq!(registry.remove(first), None);

        let replacement = registry.reserve();
        assert_eq!(replacement.lo(), first.lo());
        assert_ne!(replacement.hi(), first.hi());
        assert!(registry.resolve(first).is_none());
    }
}
