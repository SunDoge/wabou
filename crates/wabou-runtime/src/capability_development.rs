//! Cargo-based development lifecycle for trusted capability modules.
use crate::{HostService, HostServiceContext, capability_module::CapabilityModule};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
    time::Duration,
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
#[derive(Clone, Default)]
pub struct DevelopmentStatus {
    /// Number of completed build/reload attempts since service construction.
    pub attempts: u64,
    /// Error from the last attempt; None means it installed successfully.
    pub error: Option<String>,
}
/// A host-lifetime Cargo builder and source watcher. Failed builds leave code live.
pub struct CapabilityDevelopment {
    module: CapabilityModule,
    project: ModuleProject,
    worker: Mutex<Option<Worker>>,
    status: Arc<Mutex<DevelopmentStatus>>,
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
        // Capture before compiling so edits made during compilation are not lost.
        let baseline = fingerprint(
            &self.project.watch,
            self.project.target_directory.as_deref(),
        )?;
        let initial =
            build(&self.project, &stop).and_then(|path| unsafe { self.module.reload(&path) });
        {
            let mut status = self.status.lock().unwrap_or_else(|e| e.into_inner());
            status.attempts += 1;
            status.error = initial.as_ref().err().cloned();
        }
        initial?;
        let module = self.module.clone();
        let project = self.project.clone();
        let stopped = stop.clone();
        let status = self.status.clone();
        let thread = std::thread::Builder::new().name("wabou-module-build".into()).spawn(move || {
            let mut previous = baseline;
            loop {
                let (lock, wake) = &*stopped;
                let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
                if *guard { break; }
                let (guard, _) = wake.wait_timeout(guard, Duration::from_millis(200)).unwrap_or_else(|e| e.into_inner());
                if *guard { break; }
                drop(guard);
                let next = match fingerprint(&project.watch, project.target_directory.as_deref()) {
                    Ok(next) => next,
                    Err(error) => { tracing::warn!(%error, "module source scan failed"); continue; }
                };
                if next == previous { continue; }
                previous = next;
                let result = build(&project, &stopped).and_then(|path| unsafe { module.reload(&path) });
                {
                    let mut status = status.lock().unwrap_or_else(|e| e.into_inner());
                    status.attempts += 1;
                    status.error = result.as_ref().err().cloned();
                }
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
fn fingerprint(paths: &[PathBuf], excluded: Option<&Path>) -> Result<u64, String> {
    fn normalize(path: &Path) -> std::io::Result<PathBuf> {
        path.canonicalize().or_else(|_| std::path::absolute(path))
    }
    let excluded = excluded
        .map(normalize)
        .transpose()
        .map_err(|e| e.to_string())?;
    fn scan(path: &Path, hash: &mut DefaultHasher, excluded: Option<&Path>) -> std::io::Result<()> {
        if let Some(excluded) = excluded {
            if normalize(path)? == excluded {
                return Ok(());
            }
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
