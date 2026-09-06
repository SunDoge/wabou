//! Application host for the Winit + Taffy + Vello Hybrid backend.

use std::{collections::HashMap, sync::Arc};

use snafu::ResultExt;
use vello::peniko::Color;

use crate::{Applier, ImageResourceStore, JsRuntime, JsRuntimeOptions};
use legacy_shell::{FrameSource, Widget, WidgetFactory, WindowOptions};

/// Builder for the independent Winit + Taffy + Vello Hybrid application path.
///
/// It consumes the same JavaScript bundle and binary mutation protocol as the
/// GPUI host. The distinct type keeps backend-specific native widgets explicit
/// while the second backend is brought to feature parity.
pub struct WinitHostBuilder {
    window: WindowOptions,
    additional_windows: Vec<WindowOptions>,
    base_color: Color,
    widget_factories: HashMap<String, WidgetFactory>,
    image_resources: ImageResourceStore,
    js_runtime_options: JsRuntimeOptions,
}

impl Default for WinitHostBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WinitHostBuilder {
    /// Construct a host with the standard Winit native-widget registry.
    pub fn new() -> Self {
        Self::with_image_resources(ImageResourceStore::default())
    }

    /// Construct a host whose Rust code shares ownership of image resources.
    pub fn with_image_resources(image_resources: ImageResourceStore) -> Self {
        Self {
            window: WindowOptions::default(),
            additional_windows: Vec::new(),
            base_color: Color::from_rgb8(0x0f, 0x17, 0x2a),
            widget_factories: wabou_legacy_widgets::builtin_factories(),
            image_resources,
            js_runtime_options: JsRuntimeOptions::default(),
        }
    }

    /// Configure the primary native window.
    pub fn window(mut self, options: WindowOptions) -> Self {
        self.window = options;
        self
    }

    /// Add another native window running an independent copy of the bundle.
    pub fn additional_window(mut self, options: WindowOptions) -> Self {
        self.additional_windows.push(options);
        self
    }

    /// Register or replace a Winit native widget.
    pub fn widget(
        mut self,
        tag: impl Into<String>,
        factory: impl Fn() -> Box<dyn Widget> + 'static,
    ) -> Self {
        self.widget_factories.insert(tag.into(), Arc::new(factory));
        self
    }

    /// Set the physical surface clear color.
    pub fn base_color(mut self, rgba: [u8; 4]) -> Self {
        self.base_color = Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
        self
    }

    /// Configure the maximum native stack available to every QuickJS runtime.
    pub fn quickjs_stack_size(mut self, bytes: usize) -> Self {
        self.js_runtime_options = self.js_runtime_options.max_stack_size(bytes);
        self
    }

    /// Boot one runtime per window and enter the Winit event loop.
    pub fn run(self) -> crate::Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .try_init()
            .ok();

        let bundle = crate::bundle::load()?;
        let windows = std::iter::once(self.window)
            .chain(self.additional_windows)
            .collect::<Vec<_>>();
        let mut sources = Vec::with_capacity(windows.len());
        for (index, options) in windows.into_iter().enumerate() {
            let mut js = JsRuntime::new_with_options(self.js_runtime_options).context(
                crate::error::JavaScriptSnafu {
                    operation: "create JavaScript runtime",
                },
            )?;
            js.boot(&bundle).context(crate::error::JavaScriptSnafu {
                operation: "boot JavaScript bundle",
            })?;
            let mut controller = Applier::from_runtime_with_factories_and_window(
                js,
                self.widget_factories.clone(),
                self.base_color,
                legacy_shell::initial_window_resource_key(index),
            );
            controller.set_image_resource_store(self.image_resources.clone());
            sources.push((Box::new(controller) as Box<dyn FrameSource>, options));
        }
        legacy_shell::run_windows(sources).context(crate::error::WinitShellSnafu)?;
        Ok(())
    }
}
