//! Application host for the Winit + Taffy + Vello Hybrid backend.

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use vello::peniko::Color;
use wabou_bindgen::{CapabilityContract, JsonMethod};

use crate::{Applier, ImageResourceStore, JsRuntime, JsRuntimeOptions};
use legacy_shell::{FrameSource, Widget, WidgetFactory, WindowOptions};

type CapabilityInstaller = Arc<dyn Fn(&JsRuntime) -> rquickjs::Result<()>>;

const IMAGE_RESOURCES: CapabilityContract = CapabilityContract::new("imageResources", 1);
const CREATE_FILE_IMAGE: JsonMethod<CreateFileImageRequest, ImageResourceDescriptor> =
    JsonMethod::new("createFile");
const CREATE_NETWORK_IMAGE: JsonMethod<CreateNetworkImageRequest, ImageResourceDescriptor> =
    JsonMethod::new("createNetwork");
const RELEASE_IMAGE: JsonMethod<crate::ImageResourceHandle, bool> = JsonMethod::new("release");

#[derive(Deserialize)]
struct CreateFileImageRequest {
    path: String,
}

#[derive(Deserialize)]
struct CreateNetworkImageRequest {
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageResourceDescriptor {
    handle: crate::ImageResourceHandle,
    width: u32,
    height: u32,
}

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
    capabilities: Vec<CapabilityInstaller>,
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
        let builder = Self {
            window: WindowOptions::default(),
            additional_windows: Vec::new(),
            base_color: Color::from_rgb8(0x0f, 0x17, 0x2a),
            widget_factories: wabou_legacy_widgets::builtin_factories(),
            capabilities: Vec::new(),
            image_resources: image_resources.clone(),
            js_runtime_options: JsRuntimeOptions::default(),
        };
        let mounted = image_resources;
        builder.capability(IMAGE_RESOURCES, move |capability| {
            let files = mounted.clone();
            capability.json_method(CREATE_FILE_IMAGE, move |request: CreateFileImageRequest| {
                let resources = files.clone();
                async move {
                    let loader = resources.clone();
                    let handle =
                        tokio::task::spawn_blocking(move || loader.create_file(request.path))
                            .await
                            .map_err(|error| error.to_string())??;
                    let (width, height) = resources
                        .get(handle)
                        .ok_or_else(|| "created image resource did not resolve".to_owned())?
                        .dimensions();
                    Ok::<_, String>(ImageResourceDescriptor {
                        handle,
                        width,
                        height,
                    })
                }
            })?;
            let network = mounted.clone();
            capability.json_method(
                CREATE_NETWORK_IMAGE,
                move |request: CreateNetworkImageRequest| {
                    let resources = network.clone();
                    async move {
                        let handle = resources.create_network(&request.url).await?;
                        let (width, height) = resources
                            .get(handle)
                            .ok_or_else(|| "created image resource did not resolve".to_owned())?
                            .dimensions();
                        Ok::<_, String>(ImageResourceDescriptor {
                            handle,
                            width,
                            height,
                        })
                    }
                },
            )?;
            let release = mounted.clone();
            capability.json_method(RELEASE_IMAGE, move |handle: crate::ImageResourceHandle| {
                let resources = release.clone();
                async move { Ok::<_, String>(resources.remove(handle)) }
            })
        })
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

    /// Mount one versioned application capability into every window runtime.
    ///
    /// This is API-compatible with the default GPUI host: direct structured
    /// methods and opt-in JSON methods use the same generated contracts.
    pub fn capability<F>(mut self, contract: CapabilityContract, mount: F) -> Self
    where
        F: for<'js> Fn(runtime_api::NativeCapability<'js>) -> rquickjs::Result<()>
            + rquickjs::markers::ParallelSend
            + Send
            + Sync
            + 'static,
    {
        let name = contract.name().to_owned();
        let version = contract.version();
        let mount = Arc::new(mount);
        self.capabilities.push(Arc::new(move |js| {
            let mount = mount.clone();
            js.mount_capability(&name, move |ctx, object| {
                object.set("__wabouCapabilityVersion", version)?;
                mount(runtime_api::NativeCapability::from_runtime_parts(
                    ctx, object,
                ))
            })
        }));
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
            for capability in &self.capabilities {
                capability(&js).context(crate::error::JavaScriptSnafu {
                    operation: "mount JavaScript capability",
                })?;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_bindgen::HostMethod;

    #[derive(Deserialize)]
    struct DoubleRequest {
        value: u32,
    }

    #[derive(Serialize)]
    struct DoubleResponse {
        value: u32,
    }

    #[test]
    fn typed_capabilities_mount_before_the_winit_bundle_boots() {
        const CONTRACT: CapabilityContract = CapabilityContract::new("winitTest", 3);
        const DOUBLE: HostMethod<DoubleRequest, DoubleResponse> = HostMethod::new("double");
        let builder = WinitHostBuilder::new().capability(CONTRACT, |capability| {
            capability.sync_method(DOUBLE, |request: DoubleRequest| {
                Ok::<_, String>(DoubleResponse {
                    value: request.value * 2,
                })
            })
        });
        let runtime = JsRuntime::new().expect("runtime");
        for capability in &builder.capabilities {
            capability(&runtime).expect("mount capability");
        }
        let result = runtime
            .with(|ctx| {
                ctx.eval::<String, _>(
                    "JSON.stringify([__wabou_capabilities.winitTest.__wabouCapabilityVersion, __wabou_capabilities.winitTest.double({ value: 21 }).value])",
                )
            })
            .expect("invoke capability");
        assert_eq!(result, "[3,42]");
    }
}
