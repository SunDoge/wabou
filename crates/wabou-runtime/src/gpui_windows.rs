//! Application-level identities for GPUI windows.

use std::{collections::HashMap, rc::Rc};

use gpui_shell::{WindowOptions, gpui::AppContext as _};
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

type RuntimeFactory = dyn Fn(
    gpui_shell::WindowResourceKey,
    &WindowOptions,
) -> Result<crate::gpui_controller::GpuiController, String>;

/// GPUI application-level owner for native windows and their independent JS runtimes.
pub(crate) struct GpuiApplicationWindows {
    registry: GpuiWindowRegistry,
    runtime_factory: Rc<RuntimeFactory>,
    native_widget_factories: HashMap<String, gpui_shell::NativeWidgetFactory>,
    test_controller: Option<crate::test_driver::TestController>,
}

impl GpuiApplicationWindows {
    pub(crate) fn new(
        runtime_factory: Rc<RuntimeFactory>,
        native_widget_factories: HashMap<String, gpui_shell::NativeWidgetFactory>,
        test_controller: Option<crate::test_driver::TestController>,
    ) -> Rc<Self> {
        Rc::new(Self {
            registry: GpuiWindowRegistry::default(),
            runtime_factory,
            native_widget_factories,
            test_controller,
        })
    }

    pub(crate) fn reserve(&self) -> gpui_shell::WindowResourceKey {
        self.registry.reserve()
    }

    pub(crate) fn resolve(
        &self,
        key: gpui_shell::WindowResourceKey,
    ) -> Option<gpui_shell::gpui::AnyWindowHandle> {
        self.registry.resolve(key)
    }

    #[cfg(test)]
    pub(crate) fn attach(
        &self,
        key: gpui_shell::WindowResourceKey,
        handle: gpui_shell::gpui::AnyWindowHandle,
    ) -> bool {
        self.registry.attach(key, handle)
    }

    pub(crate) fn remove(
        &self,
        key: gpui_shell::WindowResourceKey,
    ) -> Option<gpui_shell::gpui::AnyWindowHandle> {
        self.registry.remove(key)
    }

    pub(crate) fn observe_native_closes(
        &self,
        cx: &gpui_shell::gpui::App,
    ) -> gpui_shell::gpui::Subscription {
        self.registry.observe_native_closes(cx)
    }

    pub(crate) fn create(
        self: &Rc<Self>,
        options: WindowOptions,
        cx: &mut gpui_shell::gpui::App,
    ) -> Result<gpui_shell::WindowResourceKey, String> {
        let key = self.reserve();
        let controller = match (self.runtime_factory)(key, &options) {
            Ok(controller) => controller,
            Err(error) => {
                self.remove(key);
                return Err(error);
            }
        };
        self.open_controller(key, controller, options, None, cx)?;
        Ok(key)
    }

    pub(crate) fn create_controller(
        &self,
        key: gpui_shell::WindowResourceKey,
        options: &WindowOptions,
    ) -> Result<crate::gpui_controller::GpuiController, String> {
        (self.runtime_factory)(key, options)
    }

    pub(crate) fn open_controller(
        self: &Rc<Self>,
        key: gpui_shell::WindowResourceKey,
        controller: crate::gpui_controller::GpuiController,
        options: WindowOptions,
        persistence: Option<gpui_shell::WindowSizePersistence>,
        cx: &mut gpui_shell::gpui::App,
    ) -> Result<gpui_shell::gpui::AnyWindowHandle, String> {
        let bounds = gpui_shell::gpui::Bounds::centered(
            None,
            gpui_shell::gpui::size(
                gpui_shell::gpui::px(options.initial_inner_size.0 as f32),
                gpui_shell::gpui::px(options.initial_inner_size.1 as f32),
            ),
            cx,
        );
        let title = options.title.clone();
        let window_host = self.clone();
        let widget_factories = self.native_widget_factories.clone();
        let test_controller = self.test_controller.clone();
        let gpui_options = gpui_shell::gpui::WindowOptions {
            window_bounds: Some(gpui_shell::gpui::WindowBounds::Windowed(bounds)),
            titlebar: options.decorations.then(Default::default),
            is_resizable: options.resizable,
            window_min_size: options.min_inner_size.map(|(width, height)| {
                gpui_shell::gpui::size(
                    gpui_shell::gpui::px(width as f32),
                    gpui_shell::gpui::px(height as f32),
                )
            }),
            window_background: if options.transparent {
                gpui_shell::gpui::WindowBackgroundAppearance::Transparent
            } else {
                gpui_shell::gpui::WindowBackgroundAppearance::Opaque
            },
            ..Default::default()
        };
        let opened = cx.open_window(gpui_options, move |window, cx| {
            window.set_window_title(&title);
            cx.new(|cx| {
                crate::GpuiRuntimeView::new(
                    controller,
                    crate::gpui_view::GpuiRuntimeViewOptions {
                        default_title: title,
                        window_size_persistence: persistence,
                        native_widget_factories: widget_factories,
                        test_controller,
                        window_key: key,
                        window_host,
                    },
                    window,
                    cx,
                )
            })
        });
        match opened {
            Ok(handle) => {
                let handle = handle.into();
                assert!(self.registry.attach(key, handle));
                Ok(handle)
            }
            Err(error) => {
                self.remove(key);
                Err(error.to_string())
            }
        }
    }
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

    pub(crate) fn observe_native_closes(
        &self,
        cx: &gpui_shell::gpui::App,
    ) -> gpui_shell::gpui::Subscription {
        let registry = self.clone();
        cx.on_window_closed(move |_, window_id| {
            let key = {
                let entries = registry.entries.borrow();
                entries.iter().find_map(|(key, handle)| {
                    handle
                        .is_some_and(|handle| handle.window_id() == window_id)
                        .then_some(key)
                })
            };
            if let Some(key) = key {
                registry.entries.borrow_mut().remove(key);
            }
        })
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
