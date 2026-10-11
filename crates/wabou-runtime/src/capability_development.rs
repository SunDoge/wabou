//! Cargo-based development lifecycle for trusted capability modules.
use crate::{HostService, HostServiceContext, capability_module::CapabilityModule};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

/// A Cargo project whose `cdylib` is replaceable while the host stays running.
#[derive(Clone)]
pub struct ModuleProject {
    /// Cargo manifest of a library target with crate-type = ["cdylib"].
    pub manifest: PathBuf,
    /// Source directories/files to watch, including relevant local dependencies.
    pub watch: Vec<PathBuf>,
    /// Optional Cargo features for the module, independent of host features.
    pub features: Vec<String>,
    /// Optional isolated Cargo output directory. None respects Cargo/mbx defaults.
    /// Useful for independent builders whose artifacts must not overwrite each other.
    pub target_directory: Option<PathBuf>,
}
impl ModuleProject {
    /// Watch this project's directory. Add local dependency roots as needed.
    pub fn new(manifest: impl Into<PathBuf>) -> Self {
        let manifest = manifest.into();
        Self {
            watch: vec![manifest.parent().unwrap_or(Path::new(".")).to_owned()],
            manifest,
            features: Vec::new(),
            target_directory: None,
        }
    }
}

struct Worker {
    stop: Arc<(Mutex<bool>, Condvar)>,
    thread: JoinHandle<()>,
}
/// Last completed build attempt, available to development diagnostics.
#[derive(Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevelopmentStatus {
    /// Number of completed build/reload attempts since service construction.
    pub attempts: u64,
    /// Error from the last attempt; None means it installed successfully.
    pub error: Option<String>,
    /// Whether Cargo is currently compiling a replacement.
    pub building: bool,
    /// Compilation and implementation replacement time of the last attempt.
    pub elapsed_ms: u64,
}
/// A host-lifetime Cargo builder and source watcher. Failed builds leave code live.
pub struct CapabilityDevelopment {
    module: CapabilityModule,
    project: ModuleProject,
    worker: Mutex<Option<Worker>>,
    status: Arc<Mutex<DevelopmentStatus>>,
    messages: Arc<Mutex<Vec<crate::HostMessageHandle>>>,
}
impl CapabilityDevelopment {
    /// Construct development support for a trusted project.
    ///
    /// # Safety
    /// Project output must obey the module ABI. This executes build scripts and
    /// loads native libraries; it is not suitable for untrusted source code.
    pub unsafe fn new(module: CapabilityModule, project: ModuleProject) -> Self {
        Self {
            module,
            project,
            worker: Mutex::new(None),
            status: Arc::new(Mutex::new(DevelopmentStatus::default())),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    /// Connect build diagnostics to every window through the existing host inbox.
    /// Register the returned producer before starting the host.
    pub fn diagnostics(&self) -> impl Fn(crate::HostMessageContext) + Send + Sync + 'static {
        let status = self.status.clone();
        let messages = self.messages.clone();
        let target = self.project.manifest.display().to_string();
        move |context| {
            let status = status.lock().unwrap_or_else(|e| e.into_inner());
            let mut messages = messages.lock().unwrap_or_else(|e| e.into_inner());
            let handle = context.messages().clone();
            let _ = handle.send(status_message(&target, &status));
            messages.push(handle);
        }
    }
    /// Read completed build/reload diagnostics.
    pub fn status(&self) -> DevelopmentStatus {
        self.status
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}
impl HostService for CapabilityDevelopment {
    fn name(&self) -> &'static str {
        "capability-development"
    }
    fn start(&self, _: &HostServiceContext) -> Result<(), String> {
        let mut worker = self.worker.lock().map_err(|e| e.to_string())?;
        if worker.is_some() {
            return Err("module watcher already started".into());
        }
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let dirty = Arc::new(AtomicBool::new(false));
        let watcher_failed = Arc::new(AtomicBool::new(false));
        let watcher = source_watcher(
            &self.project,
            stop.clone(),
            dirty.clone(),
            watcher_failed.clone(),
        );
        // Capture before compiling so edits made during compilation are not lost.
        let baseline = fingerprint(
            &self.project.watch,
            self.project.target_directory.as_deref(),
        )?;
        publish_status(&self.status, &self.messages, &self.project.manifest, None);
        let started = Instant::now();
        let initial =
            build(&self.project, &stop).and_then(|path| unsafe { self.module.reload(&path) });
        publish_status(
            &self.status,
            &self.messages,
            &self.project.manifest,
            Some((&initial, started.elapsed())),
        );
        initial?;
        let module = self.module.clone();
        let project = self.project.clone();
        let stopped = stop.clone();
        let status = self.status.clone();
        let messages = self.messages.clone();
        let thread = std::thread::Builder::new().name("wabou-module-build".into()).spawn(move || {
            let _watcher = watcher;
            let mut previous = baseline;
            loop {
                let (lock, wake) = &*stopped;
                let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
                if *guard { break; }
                let polling = _watcher.is_none() || watcher_failed.load(Ordering::Acquire);
                let guard = if !dirty.swap(false, Ordering::AcqRel) {
                    let (guard, timed) = wake.wait_timeout(guard, if polling { Duration::from_millis(200) } else { Duration::from_secs(30) }).unwrap_or_else(|e| e.into_inner());
                    if *guard { break; }
                    if !timed.timed_out() && !dirty.swap(false, Ordering::AcqRel) { continue; }
                    guard
                } else { guard };
                if *guard { break; }
                drop(guard);
                // Collapse editor bursts; content fingerprints prevent duplicate builds.
                std::thread::sleep(Duration::from_millis(75));
                let next = match fingerprint(&project.watch, project.target_directory.as_deref()) {
                    Ok(next) => next,
                    Err(error) => { tracing::warn!(%error, "module source scan failed"); continue; }
                };
                if next == previous { continue; }
                previous = next;
                publish_status(&status, &messages, &project.manifest, None);
                let started = Instant::now();
                let result = build(&project, &stopped).and_then(|path| unsafe { module.reload(&path) });
                publish_status(&status, &messages, &project.manifest, Some((&result, started.elapsed())));
                match result {
                    Ok(()) => tracing::info!(manifest = %project.manifest.display(), "capability implementation replaced"),
                    Err(error) => tracing::warn!(%error, "module build/reload failed; previous implementation retained"),
                }
            }
        }).map_err(|e| e.to_string())?;
        *worker = Some(Worker { stop, thread });
        Ok(())
    }
    fn shutdown(&self) -> Result<(), String> {
        if let Some(worker) = self.worker.lock().map_err(|e| e.to_string())?.take() {
            let (lock, wake) = &*worker.stop;
            *lock.lock().map_err(|e| e.to_string())? = true;
            wake.notify_all();
            worker
                .thread
                .join()
                .map_err(|_| "module watcher panicked".to_owned())?;
        }
        Ok(())
    }
}
impl Drop for CapabilityDevelopment {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
fn status_message(target: &str, status: &DevelopmentStatus) -> crate::HostMessage {
    // Bound transported diagnostics; retain the complete rustc output in status().
    let error = status
        .error
        .as_ref()
        .map(|value| value.chars().take(8000).collect::<String>());
    crate::HostMessage::str("wabou:reload-status", serde_json::json!({
        "source": "rust", "target": target, "attempt": status.attempts,
        "phase": if status.building { "building" } else if error.is_some() { "error" } else { "ready" },
        "elapsedMs": status.elapsed_ms, "error": error,
    }).to_string())
}
fn publish_status(
    state: &Mutex<DevelopmentStatus>,
    messages: &Mutex<Vec<crate::HostMessageHandle>>,
    target: &Path,
    result: Option<(&Result<(), String>, Duration)>,
) {
    let mut status = state.lock().unwrap_or_else(|e| e.into_inner());
    status.building = result.is_none();
    if let Some((result, elapsed)) = result {
        status.attempts += 1;
        status.error = result.as_ref().err().cloned();
        status.elapsed_ms = elapsed.as_millis().min(u64::MAX as u128) as u64;
    }
    let message = status_message(&target.display().to_string(), &status);
    messages
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .retain(|handle| {
            !matches!(
                handle.send(message.clone()),
                Err(crate::HostMessageError::Disconnected)
            )
        });
    tracing::info!(target: "hmr", source = "rust", manifest = %target.display(), building = status.building, attempt = status.attempts, elapsed_ms = status.elapsed_ms, error = ?status.error, "capability development status");
}

fn source_watcher(
    project: &ModuleProject,
    stop: Arc<(Mutex<bool>, Condvar)>,
    dirty: Arc<AtomicBool>,
    failed: Arc<AtomicBool>,
) -> Option<notify::RecommendedWatcher> {
    use notify::Watcher;
    let normalize = |path: &Path| path.canonicalize().or_else(|_| std::path::absolute(path));
    let paths = project
        .watch
        .iter()
        .map(|path| normalize(path))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let excluded = project
        .target_directory
        .as_deref()
        .map(normalize)
        .transpose()
        .ok()?;
    let watched = paths.clone();
    let result = (|| -> notify::Result<_> {
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                let relevant = match event {
                    Ok(event) if matches!(event.kind, notify::EventKind::Access(_)) => false,
                    Ok(event) => event.paths.iter().any(|path| {
                        watched.iter().any(|root| {
                            path.strip_prefix(root).is_ok_and(|relative| {
                                !relative.components().any(|component| {
                                    matches!(
                                        component.as_os_str().to_str(),
                                        Some("target" | ".git" | "node_modules" | "dist")
                                    )
                                })
                            })
                        }) && !excluded.as_ref().is_some_and(|root| path.starts_with(root))
                    }),
                    Err(error) => {
                        tracing::warn!(%error, "source watcher failed; falling back to polling");
                        failed.store(true, Ordering::Release);
                        true
                    }
                };
                if relevant {
                    let _guard = stop.0.lock().unwrap_or_else(|e| e.into_inner());
                    dirty.store(true, Ordering::Release);
                    stop.1.notify_all();
                }
            })?;
        let mut roots = std::collections::BTreeSet::new();
        for path in paths {
            let directory = if path.is_dir() {
                path
            } else {
                path.parent().unwrap_or(Path::new(".")).to_owned()
            };
            roots.insert(directory);
        }
        for root in roots {
            watcher.watch(&root, notify::RecursiveMode::Recursive)?;
        }
        Ok(watcher)
    })();
    match result {
        Ok(watcher) => Some(watcher),
        Err(error) => {
            tracing::warn!(%error, "native source watching unavailable; falling back to polling");
            None
        }
    }
}

fn fingerprint(paths: &[PathBuf], excluded: Option<&Path>) -> Result<u64, String> {
    fn normalize(path: &Path) -> std::io::Result<PathBuf> {
        path.canonicalize().or_else(|_| std::path::absolute(path))
    }
    let excluded = excluded
        .map(normalize)
        .transpose()
        .map_err(|e| e.to_string())?;
    fn scan(path: &Path, hash: &mut DefaultHasher, excluded: Option<&Path>) -> std::io::Result<()> {
        if let Some(excluded) = excluded
            && normalize(path)? == excluded
        {
            return Ok(());
        }
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                path.hash(hash);
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        if metadata.is_symlink() {
            return Ok(());
        }
        path.hash(hash);
        if metadata.is_dir() {
            let mut children = std::fs::read_dir(path)?
                .map(|e| e.map(|e| e.path()))
                .collect::<Result<Vec<_>, _>>()?;
            children.sort();
            for child in children {
                if matches!(
                    child.file_name().and_then(|v| v.to_str()),
                    Some("target" | ".git" | "node_modules" | "dist")
                ) {
                    continue;
                }
                scan(&child, hash, excluded)?;
            }
        } else {
            // Content hashing also detects same-size edits on coarse clocks.
            std::fs::read(path)?.hash(hash);
        }
        Ok(())
    }
    let mut hash = DefaultHasher::new();
    for path in paths {
        scan(path, &mut hash, excluded.as_deref()).map_err(|e| e.to_string())?;
    }
    Ok(hash.finish())
}
fn build(project: &ModuleProject, stop: &(Mutex<bool>, Condvar)) -> Result<PathBuf, String> {
    let _measurement = wabou_telemetry::span("reload", "cargo.build");
    // File-backed Cargo JSON avoids full pipes blocking compilation and lets us
    // resolve target directories, names and platforms from actual artifacts.
    let mut log = tempfile::tempfile().map_err(|e| e.to_string())?;
    let mut cargo = Command::new("cargo");
    cargo
        .args(["build", "--lib", "--message-format=json", "--manifest-path"])
        .arg(&project.manifest)
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(Stdio::inherit());
    if !project.features.is_empty() {
        cargo.arg("--features").arg(project.features.join(","));
    }
    if let Some(directory) = &project.target_directory {
        cargo.arg("--target-dir").arg(directory);
    }
    let mut child = cargo.spawn().map_err(|e| e.to_string())?;
    let success = loop {
        if *stop.0.lock().map_err(|e| e.to_string())? {
            let _ = child.kill();
            let _ = child.wait();
            return Err("module build cancelled".into());
        }
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => break status.success(),
            None => std::thread::sleep(Duration::from_millis(25)),
        }
    };
    use std::io::{Seek, SeekFrom};
    log.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    let mut text = String::new();
    log.read_to_string(&mut text).map_err(|e| e.to_string())?;
    if !success {
        return Err(compilation_error(&text));
    }
    let manifest = project.manifest.canonicalize().map_err(|e| e.to_string())?;
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if value["reason"] != "compiler-artifact"
            || value["manifest_path"].as_str() != manifest.to_str()
        {
            continue;
        }
        if !value["target"]["crate_types"]
            .as_array()
            .is_some_and(|types| types.iter().any(|v| v == "cdylib"))
        {
            continue;
        }
        if let Some(files) = value["filenames"].as_array() {
            for file in files.iter().filter_map(|v| v.as_str()) {
                if file.ends_with(std::env::consts::DLL_SUFFIX) {
                    return Ok(PathBuf::from(file));
                }
            }
        }
    }
    Err("Cargo produced no cdylib for the module manifest".into())
}

// Preserve rustc's file/line diagnostics for the devtools/status consumer. Cargo
// writes compiler messages to stdout in JSON mode, rather than stderr.
fn compilation_error(log: &str) -> String {
    let diagnostics = log
        .lines()
        .filter_map(|line| {
            let value: serde_json::Value = serde_json::from_str(line).ok()?;
            (value["reason"] == "compiler-message" && value["message"]["level"] == "error")
                .then(|| value["message"]["rendered"].as_str().map(str::to_owned))
                .flatten()
        })
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        "Cargo module compilation failed; see Cargo stderr for details".into()
    } else {
        format!(
            "Cargo module compilation failed:\n{}",
            diagnostics.join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostics_replay_latest_status_and_publish_build_failure_and_recovery() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let module = CapabilityModule::new(1, Arc::new(|_, _| Ok(Vec::new())));
        let service = unsafe {
            CapabilityDevelopment::new(module, ModuleProject::new("functions/Cargo.toml"))
        };
        let (handle, inbox) = crate::host_message::host_message_channel(16);
        let context = crate::HostMessageContext::new(
            wabou_shell_api::WindowResourceKey::from_parts(1, 1).unwrap(),
            handle,
            tokio_util::sync::CancellationToken::new(),
            runtime.handle().clone(),
            Arc::new(crate::host_message::HostTaskTracker::default()),
        );
        let failed: Result<(), String> =
            Err("error[E0425]: missing value\n --> src/lib.rs:8:3".into());
        publish_status(
            &service.status,
            &service.messages,
            &service.project.manifest,
            Some((&failed, Duration::from_millis(123))),
        );
        service.diagnostics()(context);
        let decode = |message: crate::HostMessage| {
            assert_eq!(message.topic, "wabou:reload-status");
            let crate::HostMessagePayload::Str(json) = message.payload else {
                panic!("JSON status")
            };
            serde_json::from_str::<serde_json::Value>(&json).unwrap()
        };
        let replay = decode(inbox.drain_batch().pop().unwrap());
        assert_eq!(replay["phase"], "error");
        assert_eq!(replay["elapsedMs"], 123);
        assert!(replay["error"].as_str().unwrap().contains("src/lib.rs:8:3"));
        publish_status(
            &service.status,
            &service.messages,
            &service.project.manifest,
            None,
        );
        assert_eq!(
            decode(inbox.drain_batch().pop().unwrap())["phase"],
            "building"
        );
        publish_status(
            &service.status,
            &service.messages,
            &service.project.manifest,
            Some((&Ok(()), Duration::from_millis(20))),
        );
        let ready = decode(inbox.drain_batch().pop().unwrap());
        assert_eq!(ready["phase"], "ready");
        assert_eq!(ready["attempt"], 2);
        assert!(ready["error"].is_null());
    }

    #[test]
    fn compiler_errors_preserve_source_locations_without_json_noise() {
        let log = [
            serde_json::json!({"reason":"compiler-message", "message":{"level":"warning","rendered":"unused variable"}}).to_string(),
            serde_json::json!({"reason":"compiler-message", "message":{"level":"error","rendered":"error[E0425]: missing value\n --> src/lib.rs:8:3\n"}}).to_string(),
            serde_json::json!({"reason":"build-finished","success":false}).to_string(),
        ].join("\n");
        let error = compilation_error(&log);
        assert!(error.contains("E0425"));
        assert!(error.contains("src/lib.rs:8:3"));
        assert!(!error.contains("unused variable"));
        assert!(!error.contains("build-finished"));
        assert!(compilation_error("").contains("stderr"));
    }
    #[test]
    fn custom_target_output_is_not_a_watched_source_change() {
        let project = tempfile::tempdir().unwrap();
        let source = project.path().join("lib.rs");
        std::fs::write(&source, "version one").unwrap();
        let output = project.path().join("build-cache");
        let paths = vec![project.path().to_owned()];
        let before = fingerprint(&paths, Some(&output)).unwrap();
        std::fs::create_dir(&output).unwrap();
        std::fs::write(output.join("library.so"), "compiled artifact").unwrap();
        assert_eq!(fingerprint(&paths, Some(&output)).unwrap(), before);
        std::fs::write(&source, "version two").unwrap();
        assert_ne!(fingerprint(&paths, Some(&output)).unwrap(), before);
    }
}
