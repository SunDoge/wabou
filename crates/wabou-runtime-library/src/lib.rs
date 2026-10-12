//! Precompiled resident runtime; applications compile only launch/capability SDKs.
use std::sync::Arc;
use wabou_backend_vello_hybrid::{VelloHybridHostBuilder, WindowOptions};
use wabou_launch::{Implementation, LAUNCH_VERSION, LaunchConfig, LaunchResult, UiSource};
#[cfg(feature = "development")]
use wabou_runtime::capability_development::{CapabilityDevelopment, ModuleProject};
use wabou_runtime::{
    capability_files::CapabilityFiles, capability_module::CapabilityModule,
    capability_state::CapabilityState, capability_tasks::CapabilityTasks,
    capability_tcp::CapabilityTcp,
};

/// ABI-checked startup entry. The runtime stays resident for process lifetime.
/// Native library loading is trusted; callers must honor wabou-launch's contract.
#[stabby::export]
pub extern "C" fn wabou_runtime_launch(config: stabby::string::String) -> LaunchResult {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_config(serde_json::from_str(&config).map_err(|e| e.to_string())?)
    }))
    .unwrap_or_else(|_| Err("runtime startup panicked".into()));
    result
        .map_err(|error| stabby::string::String::from(error.as_str()))
        .into()
}
/// Run a trusted application on the main thread using the linked resident runtime.
pub fn run_config(config: LaunchConfig) -> Result<(), String> {
    unsafe { run_linked(config, &[]) }
}

/// Run with trusted, statically linked module tables. Matching entries bypass dynamic loading.
/// Registration keys are `capability:<name>`, `widget:<tag>` and `plugin:<name>`;
/// namespaces remain independent even when user-provided names coincide.
///
/// # Safety
/// Tables must honor the ABI and their code must remain resident for the process lifetime.
pub unsafe fn run_linked(
    config: LaunchConfig,
    linked: &[(String, wabou_capability_abi::ModuleApi)],
) -> Result<(), String> {
    let _trace_export = wabou_telemetry::ExportOnDrop;
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
            .decorations(config.decorations)
            .initial_inner_size(config.width, config.height),
    );
    host = match config.source {
        UiSource::Bundle { path } => {
            host.javascript_source(std::fs::read_to_string(path).map_err(|e| e.to_string())?)
        }
        #[cfg(feature = "development")]
        UiSource::Vite { url, entry } => host.vite_source(url, entry),
        #[cfg(not(feature = "development"))]
        UiSource::Vite { .. } => return Err("this release runtime does not support Vite".into()),
    };
    let mut widget_tags = std::collections::HashSet::new();
    let mut capability_names = std::collections::HashSet::new();
    let mut plugin_names = std::collections::HashSet::new();
    for plugin in config.plugins {
        use wabou_host_api::plugin::{ExportKind, PLUGIN_CONTRACT_VERSION};
        if plugin.name.trim().is_empty() || !plugin_names.insert(plugin.name.clone()) {
            return Err("plugin names must be nonempty and unique".into());
        }
        let root = CapabilityModule::new(
            PLUGIN_CONTRACT_VERSION,
            Arc::new(|_, _| Err("plugin root has no host state".into())),
        );
        let mut capabilities = Vec::new();
        for capability in plugin.capabilities {
            if capability.name.trim().is_empty()
                || !capability_names.insert(capability.name.clone())
            {
                return Err("capability names must be nonempty and unique".into());
            }
            let state = CapabilityState::default();
            let tasks = CapabilityTasks::new(executor.handle().clone());
            let completion = tasks.completion_notification();
            let files = CapabilityFiles::new(resource_directory.clone(), tasks.clone());
            let tcp = CapabilityTcp::new(tasks.clone());
            let module = root
                .select_export(
                    ExportKind::Capability,
                    &capability.name,
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
                )?
                .with_completion_notification(completion);
            capabilities.push((capability, module));
        }
        let mut widgets = Vec::new();
        for widget in plugin.widgets {
            if widget.tag.trim().is_empty() || !widget_tags.insert(widget.tag.clone()) {
                return Err("native widget tags must be nonempty and unique".into());
            }
            let module = root.select_export(
                ExportKind::Widget,
                &widget.tag,
                widget.version,
                Arc::new(|operation, _| {
                    Err(format!("widget host operation unavailable: {operation}"))
                }),
            )?;
            widgets.push((widget, module));
        }
        if let Some((_, api)) = linked
            .iter()
            .find(|(name, _)| name == &format!("plugin:{}", plugin.name))
        {
            unsafe {
                root.install_linked(*api)?;
            }
        } else {
            match plugin.implementation {
                Implementation::Library { path } => unsafe {
                    root.reload(&path)?;
                },
                #[cfg(not(feature = "development"))]
                Implementation::Project { .. } => {
                    return Err("this release runtime requires built plugin libraries".into());
                }
                #[cfg(feature = "development")]
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
                    let development = unsafe { CapabilityDevelopment::new(root, project) };
                    host = host
                        .host_message_producer(development.diagnostics())
                        .service(development);
                }
            }
        }
        for (capability, module) in capabilities {
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
        for (widget, module) in widgets {
            host = host.widget(widget.tag, move || {
                Box::new(
                    wabou_shell_vello::widgets::hot_reload::ReloadableWidget::new(
                        module.clone(),
                        widget.initial_state.clone(),
                    ),
                )
            });
        }
    }
    for widget in config.widgets {
        if widget.tag.trim().is_empty() || !widget_tags.insert(widget.tag.clone()) {
            return Err("native widget tags must be nonempty and unique".into());
        }
        let module = CapabilityModule::new(
            widget.version,
            Arc::new(|operation, _| Err(format!("widget host operation unavailable: {operation}"))),
        );
        if let Some((_, api)) = linked
            .iter()
            .find(|(name, _)| name == &format!("widget:{}", widget.tag))
        {
            unsafe {
                module.install_linked(*api)?;
            }
        } else {
            match widget.implementation {
                Implementation::Library { path } => unsafe {
                    module.reload(&path)?;
                },
                #[cfg(not(feature = "development"))]
                Implementation::Project { .. } => {
                    return Err("this release runtime requires built widget libraries".into());
                }
                #[cfg(feature = "development")]
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
        host = host.widget(widget.tag, move || {
            Box::new(
                wabou_shell_vello::widgets::hot_reload::ReloadableWidget::new(
                    module.clone(),
                    widget.initial_state.clone(),
                ),
            )
        });
    }
    for capability in config.capabilities {
        if capability.name.trim().is_empty() || !capability_names.insert(capability.name.clone()) {
            return Err("capability names must be nonempty and unique".into());
        }
        // Each capability receives a distinct host-owned durable data namespace.
        // Files and tasks are resident services, not objects from module code.
        let state = CapabilityState::default();
        let tasks = CapabilityTasks::new(executor.handle().clone());
        let completion = tasks.completion_notification();
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
        let module = module.with_completion_notification(completion);
        if let Some((_, api)) = linked
            .iter()
            .find(|(name, _)| name == &format!("capability:{}", capability.name))
        {
            unsafe {
                module.install_linked(*api)?;
            }
        } else {
            match capability.implementation {
                Implementation::Library { path } => unsafe {
                    module.reload(&path)?;
                },
                #[cfg(not(feature = "development"))]
                Implementation::Project { .. } => {
                    return Err("this release runtime requires built capability libraries".into());
                }
                #[cfg(feature = "development")]
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
