//! Precompiled resident runtime; applications compile only launch/capability SDKs.
use std::sync::Arc;
use wabou_backend_vello_hybrid::{VelloHybridHostBuilder, WindowOptions};
use wabou_launch::{Implementation, LAUNCH_VERSION, LaunchConfig, LaunchResult, UiSource};
use wabou_runtime::{
    capability_development::{CapabilityDevelopment, ModuleProject},
    capability_files::CapabilityFiles,
    capability_module::CapabilityModule,
    capability_state::CapabilityState,
    capability_tasks::CapabilityTasks,
    capability_tcp::CapabilityTcp,
};

/// ABI-checked startup entry. The runtime stays resident for process lifetime.
/// Native library loading is trusted; callers must honor wabou-launch's contract.
#[stabby::export]
pub extern "C" fn wabou_runtime_launch(config: stabby::string::String) -> LaunchResult {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&config)))
        .unwrap_or_else(|_| Err("runtime startup panicked".into()));
    result
        .map_err(|error| stabby::string::String::from(error.as_str()))
        .into()
}
fn run(config: &str) -> Result<(), String> {
    run_config(serde_json::from_str(config).map_err(|e| e.to_string())?)
}

/// Run a trusted application on the main thread using the linked resident runtime.
pub fn run_config(config: LaunchConfig) -> Result<(), String> {
    unsafe { run_linked(config, &[]) }
}

/// Run with trusted, statically linked capability tables. Matching names bypass dynamic loading.
///
/// # Safety
/// Tables must honor the ABI and their code must remain resident for the process lifetime.
pub unsafe fn run_linked(
    config: LaunchConfig,
    linked: &[(String, wabou_capability_abi::ModuleApi)],
) -> Result<(), String> {
    if config.version != LAUNCH_VERSION {
        return Err("incompatible runtime launch contract".into());
    }
    if config.width == 0 || config.height == 0 {
        return Err("window dimensions must be positive".into());
    }
    // One executor for resident background work across all windows/modules.
    let executor = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let resource_directory = match config.resource_directory {
        Some(path) => path,
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };
    let mut host = VelloHybridHostBuilder::new().window(
        WindowOptions::new()
            .title(config.title)
            .initial_inner_size(config.width, config.height),
    );
    host = match config.source {
        UiSource::Bundle { path } => {
            host.javascript_source(std::fs::read_to_string(path).map_err(|e| e.to_string())?)
        }
        UiSource::Vite { url, entry } => host.vite_source(url, entry),
    };
    for capability in config.capabilities {
        // Each capability receives a distinct host-owned durable data namespace.
        // Files and tasks are resident services, not objects from module code.
        let state = CapabilityState::default();
        let tasks = CapabilityTasks::new(executor.handle().clone());
        let files = CapabilityFiles::new(resource_directory.clone(), tasks.clone());
        let tcp = CapabilityTcp::new(tasks.clone());
        let module = CapabilityModule::new(
            capability.version,
            Arc::new(move |operation, request| {
                if operation.starts_with("task.") {
                    tasks.call(operation, request)
                } else if operation.starts_with("file.") {
                    files.call(operation, request)
                } else if operation.starts_with("tcp.") {
                    tcp.call(operation, request)
                } else {
                    state.call(operation, request)
                }
            }),
        );
        if let Some((_, api)) = linked.iter().find(|(name, _)| name == &capability.name) {
            unsafe {
                module.install_linked(*api)?;
            }
        } else {
            match capability.implementation {
                Implementation::Library { path } => unsafe {
                    module.reload(&path)?;
                },
                Implementation::Project {
                    manifest,
                    watch,
                    features,
                    target_directory,
                } => {
                    let mut project = ModuleProject::new(manifest);
                    project.watch.extend(watch);
                    project.features = features;
                    project.target_directory = target_directory;
                    let development =
                        unsafe { CapabilityDevelopment::new(module.clone(), project) };
                    host = host
                        .host_message_producer(development.diagnostics())
                        .service(development);
                }
            }
        }
        host = host.capability_named(capability.name, capability.version, move |native| {
            for method in &capability.methods {
                native.module_method_by_name(
                    &method.name,
                    method.has_request,
                    method.json,
                    module.clone(),
                )?;
            }
            Ok(())
        });
    }
    host.run().map_err(|e| e.to_string())
}
