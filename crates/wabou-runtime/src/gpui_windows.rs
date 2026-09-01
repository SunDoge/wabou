//! Application-level identities for GPUI windows.

use std::{collections::HashMap, rc::Rc};

use slotmap::{Key as _, KeyData, SlotMap};
use wabou_shell::{WindowOptions, gpui::AppContext as _};

slotmap::new_key_type! {
    struct GpuiWindowSlot;
}

/// Shared registry separating stable Wabou window identities from GPUI's
/// process-local handles.
#[derive(Clone, Default)]
pub(crate) struct GpuiWindowRegistry {
    entries: std::rc::Rc<
        std::cell::RefCell<SlotMap<GpuiWindowSlot, Option<wabou_shell::gpui::AnyWindowHandle>>>,
    >,
}

type RuntimeFactory = dyn Fn(
    wabou_shell::WindowResourceKey,
    &WindowOptions,
) -> Result<crate::gpui_controller::GpuiController, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GpuiWindowError {
    Unsupported(String),
    Platform(String),
}

impl GpuiWindowError {
    pub(crate) fn message(&self) -> &str {
        match self {
            Self::Unsupported(message) | Self::Platform(message) => message,
        }
    }
}

impl std::fmt::Display for GpuiWindowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

/// GPUI application-level owner for native windows and their independent JS runtimes.
pub(crate) struct GpuiApplicationWindows {
    registry: GpuiWindowRegistry,
    runtime_factory: Rc<RuntimeFactory>,
    native_widget_factories: HashMap<String, wabou_shell::NativeWidgetFactory>,
    test_controller: Option<crate::test_driver::TestController>,
}

impl GpuiApplicationWindows {
    pub(crate) fn new(
        runtime_factory: Rc<RuntimeFactory>,
        native_widget_factories: HashMap<String, wabou_shell::NativeWidgetFactory>,
        test_controller: Option<crate::test_driver::TestController>,
    ) -> Rc<Self> {
        Rc::new(Self {
            registry: GpuiWindowRegistry::default(),
            runtime_factory,
            native_widget_factories,
            test_controller,
        })
    }

    pub(crate) fn reserve(&self) -> wabou_shell::WindowResourceKey {
        self.registry.reserve()
    }

    pub(crate) fn resolve(
        &self,
        key: wabou_shell::WindowResourceKey,
    ) -> Option<wabou_shell::gpui::AnyWindowHandle> {
        self.registry.resolve(key)
    }

    #[cfg(test)]
    pub(crate) fn attach(
        &self,
        key: wabou_shell::WindowResourceKey,
        handle: wabou_shell::gpui::AnyWindowHandle,
    ) -> bool {
        self.registry.attach(key, handle)
    }

    pub(crate) fn remove(
        &self,
        key: wabou_shell::WindowResourceKey,
    ) -> Option<wabou_shell::gpui::AnyWindowHandle> {
        self.registry.remove(key)
    }

    pub(crate) fn observe_native_closes(
        &self,
        cx: &wabou_shell::gpui::App,
    ) -> wabou_shell::gpui::Subscription {
        self.registry.observe_native_closes(cx)
    }

    pub(crate) fn create(
        self: &Rc<Self>,
        options: WindowOptions,
        cx: &mut wabou_shell::gpui::App,
    ) -> Result<wabou_shell::WindowResourceKey, GpuiWindowError> {
        validate_gpui_window_options(&options)?;
        let key = self.reserve();
        let controller = match (self.runtime_factory)(key, &options) {
            Ok(controller) => controller,
            Err(error) => {
                self.remove(key);
                return Err(GpuiWindowError::Platform(error));
            }
        };
        if let Err(error) = self.open_controller(key, controller, options, None, cx) {
            self.remove(key);
            return Err(error);
        }
        Ok(key)
    }

    pub(crate) fn create_controller(
        &self,
        key: wabou_shell::WindowResourceKey,
        options: &WindowOptions,
    ) -> Result<crate::gpui_controller::GpuiController, String> {
        (self.runtime_factory)(key, options)
    }

    pub(crate) fn open_controller(
        self: &Rc<Self>,
        key: wabou_shell::WindowResourceKey,
        controller: crate::gpui_controller::GpuiController,
        options: WindowOptions,
        persistence: Option<wabou_shell::WindowSizePersistence>,
        cx: &mut wabou_shell::gpui::App,
    ) -> Result<wabou_shell::gpui::AnyWindowHandle, GpuiWindowError> {
        validate_gpui_window_options(&options)?;
        let bounds = wabou_shell::gpui::Bounds::centered(
            None,
            wabou_shell::gpui::size(
                wabou_shell::gpui::px(options.initial_inner_size.0 as f32),
                wabou_shell::gpui::px(options.initial_inner_size.1 as f32),
            ),
            cx,
        );
        let title = options.title.clone();
        let window_host = self.clone();
        let widget_factories = self.native_widget_factories.clone();
        let test_controller = self.test_controller.clone();
        let gpui_options = project_gpui_window_options(&options, bounds);
        let opened = cx.open_window(gpui_options, move |window, cx| {
            window.set_window_title(&title);
            cx.new(|cx| {
                crate::GpuiRuntimeView::new(
                    controller,
                    crate::gpui_view::GpuiRuntimeViewOptions {
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
                Err(GpuiWindowError::Platform(error.to_string()))
            }
        }
    }
}

fn project_gpui_window_options(
    options: &WindowOptions,
    bounds: wabou_shell::gpui::Bounds<wabou_shell::gpui::Pixels>,
) -> wabou_shell::gpui::WindowOptions {
    wabou_shell::gpui::WindowOptions {
        window_bounds: Some(wabou_shell::gpui::WindowBounds::Windowed(bounds)),
        titlebar: options.decorations.then(Default::default),
        kind: match options.window_level {
            wabou_shell::WindowLevel::AlwaysOnTop => wabou_shell::gpui::WindowKind::PopUp,
            wabou_shell::WindowLevel::Normal | wabou_shell::WindowLevel::AlwaysOnBottom => {
                wabou_shell::gpui::WindowKind::Normal
            }
        },
        app_owns_titlebar_drag: !options.decorations,
        is_resizable: options.resizable,
        window_min_size: options.min_inner_size.map(|(width, height)| {
            wabou_shell::gpui::size(
                wabou_shell::gpui::px(width as f32),
                wabou_shell::gpui::px(height as f32),
            )
        }),
        window_background: match options.background {
            wabou_shell::WindowBackground::Opaque => {
                wabou_shell::gpui::WindowBackgroundAppearance::Opaque
            }
            wabou_shell::WindowBackground::Transparent => {
                wabou_shell::gpui::WindowBackgroundAppearance::Transparent
            }
            wabou_shell::WindowBackground::Blurred => {
                wabou_shell::gpui::WindowBackgroundAppearance::Blurred
            }
            wabou_shell::WindowBackground::Mica => {
                wabou_shell::gpui::WindowBackgroundAppearance::MicaBackdrop
            }
            wabou_shell::WindowBackground::MicaAlt => {
                wabou_shell::gpui::WindowBackgroundAppearance::MicaAltBackdrop
            }
        },
        window_decorations: Some(if options.decorations {
            wabou_shell::gpui::WindowDecorations::Server
        } else {
            wabou_shell::gpui::WindowDecorations::Client
        }),
        ..Default::default()
    }
}

pub(crate) fn validate_gpui_window_options(options: &WindowOptions) -> Result<(), GpuiWindowError> {
    if options.window_level == wabou_shell::WindowLevel::AlwaysOnBottom {
        return Err(GpuiWindowError::Unsupported(
            "GPUI-CE does not expose an always-on-bottom native window level".to_owned(),
        ));
    }
    if options.input_mode == wabou_shell::WindowInputMode::Passthrough {
        return Err(GpuiWindowError::Unsupported(
            "GPUI-CE does not expose native pointer-passthrough windows; the request was not applied"
                .to_owned(),
        ));
    }
    Ok(())
}

impl GpuiWindowRegistry {
    pub(crate) fn reserve(&self) -> wabou_shell::WindowResourceKey {
        let key = self.entries.borrow_mut().insert(None);
        wabou_shell::WindowResourceKey::from_ffi(key.data().as_ffi())
            .expect("SlotMap generated an invalid window resource key")
    }

    pub(crate) fn attach(
        &self,
        key: wabou_shell::WindowResourceKey,
        handle: wabou_shell::gpui::AnyWindowHandle,
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
        key: wabou_shell::WindowResourceKey,
    ) -> Option<wabou_shell::gpui::AnyWindowHandle> {
        let key = GpuiWindowSlot::from(KeyData::from_ffi(key.as_ffi()));
        self.entries.borrow().get(key).copied().flatten()
    }

    pub(crate) fn remove(
        &self,
        key: wabou_shell::WindowResourceKey,
    ) -> Option<wabou_shell::gpui::AnyWindowHandle> {
        let key = GpuiWindowSlot::from(KeyData::from_ffi(key.as_ffi()));
        self.entries.borrow_mut().remove(key).flatten()
    }

    pub(crate) fn observe_native_closes(
        &self,
        cx: &wabou_shell::gpui::App,
    ) -> wabou_shell::gpui::Subscription {
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
    fn window_options_project_to_explicit_gpui_native_semantics() {
        let options = WindowOptions::new()
            .initial_inner_size(900, 640)
            .min_inner_size(480, 320)
            .decorations(false)
            .background(wabou_shell::WindowBackground::Blurred)
            .window_level(wabou_shell::WindowLevel::AlwaysOnTop);
        let projected = project_gpui_window_options(&options, Default::default());

        assert!(projected.titlebar.is_none());
        assert!(projected.app_owns_titlebar_drag);
        assert_eq!(projected.kind, wabou_shell::gpui::WindowKind::PopUp);
        assert_eq!(
            projected.window_decorations,
            Some(wabou_shell::gpui::WindowDecorations::Client)
        );
        assert_eq!(
            projected.window_background,
            wabou_shell::gpui::WindowBackgroundAppearance::Blurred
        );
        assert_eq!(
            projected.window_min_size,
            Some(wabou_shell::gpui::size(
                wabou_shell::gpui::px(480.0),
                wabou_shell::gpui::px(320.0)
            ))
        );
    }

    #[test]
    fn unsupported_gpui_window_semantics_fail_before_native_creation() {
        let bottom = WindowOptions::new().window_level(wabou_shell::WindowLevel::AlwaysOnBottom);
        assert!(
            validate_gpui_window_options(&bottom)
                .unwrap_err()
                .message()
                .contains("always-on-bottom")
        );

        let passthrough =
            WindowOptions::new().input_mode(wabou_shell::WindowInputMode::Passthrough);
        assert!(
            validate_gpui_window_options(&passthrough)
                .unwrap_err()
                .message()
                .contains("pointer-passthrough")
        );
        assert!(validate_gpui_window_options(&WindowOptions::new()).is_ok());
    }

    #[test]
    fn reservations_match_initial_keys_and_reject_removed_generations() {
        let registry = GpuiWindowRegistry::default();
        let first = registry.reserve();
        let second = registry.reserve();
        assert_eq!(first, wabou_shell::initial_window_resource_key(0));
        assert_eq!(second, wabou_shell::initial_window_resource_key(1));
        assert_eq!(registry.remove(first), None);

        let replacement = registry.reserve();
        assert_eq!(replacement.lo(), first.lo());
        assert_ne!(replacement.hi(), first.hi());
        assert!(registry.resolve(first).is_none());
    }
}
