//! GPUI application lifecycle extensions.
//!
//! Extensions execute inside GPUI's application context before application
//! services begin using the opened windows.

use gpui::{AnyWindowHandle, App, Global};

/// Read-only startup context shared with one GPUI application extension.
pub struct ApplicationExtensionContext<'a> {
    windows: &'a [AnyWindowHandle],
}

impl<'a> ApplicationExtensionContext<'a> {
    /// All windows opened during initial application startup.
    pub fn windows(&self) -> &'a [AnyWindowHandle] {
        self.windows
    }

    /// The primary application window, when one was configured.
    pub fn primary_window(&self) -> Option<AnyWindowHandle> {
        self.windows.first().copied()
    }
}

/// A service installed after GPUI has opened the application's initial windows.
///
/// Implementations may retain native resources and spawn foreground tasks from
/// `App::to_async`. Wabou stores the extension for the complete GPUI lifetime.
pub trait ApplicationExtension: 'static {
    /// Install callbacks and native resources for this application instance.
    fn install(
        &mut self,
        context: ApplicationExtensionContext<'_>,
        app: &mut App,
    ) -> Result<(), String>;
}

struct InstalledApplicationExtensions {
    _extensions: Vec<Box<dyn ApplicationExtension>>,
}

impl Global for InstalledApplicationExtensions {}

/// Install and retain GPUI-native application extensions.
pub fn install_application_extensions(
    mut extensions: Vec<Box<dyn ApplicationExtension>>,
    windows: &[AnyWindowHandle],
    app: &mut App,
) -> Result<(), String> {
    let context = ApplicationExtensionContext { windows };
    for extension in &mut extensions {
        extension.install(
            ApplicationExtensionContext {
                windows: context.windows,
            },
            app,
        )?;
    }
    app.set_global(InstalledApplicationExtensions {
        _extensions: extensions,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::TestAppContext;

    use super::*;

    struct Probe(Rc<Cell<bool>>);

    impl ApplicationExtension for Probe {
        fn install(
            &mut self,
            context: ApplicationExtensionContext<'_>,
            _app: &mut App,
        ) -> Result<(), String> {
            assert!(context.primary_window().is_none());
            self.0.set(true);
            Ok(())
        }
    }

    #[gpui::test]
    fn extensions_are_installed_in_gpui_application_order(cx: &mut TestAppContext) {
        let installed = Rc::new(Cell::new(false));
        cx.update(|app| {
            install_application_extensions(vec![Box::new(Probe(installed.clone()))], &[], app)
                .unwrap();
        });
        assert!(installed.get());
    }
}
