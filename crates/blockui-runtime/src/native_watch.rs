//! Development watcher implementation, separate from retained native UI state.
use super::*;

pub(super) struct Watch {
    pub(super) stop: Arc<AtomicBool>,
    pub(super) thread: Option<thread::JoinHandle<()>>,
    pub(super) ready: Arc<AtomicBool>,
    pub(super) published: Arc<std::sync::atomic::AtomicU64>,
    pub(super) superseded: Arc<std::sync::atomic::AtomicU64>,
}
impl Drop for Watch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
pub(crate) struct BuildResult {
    pub(crate) artifact: Option<PathBuf>,
    pub(crate) message: String,
}
/// Development configuration is independent of the application crate and renderer.
#[derive(Clone, Debug)]
pub struct ReloadConfig {
    pub manifest: PathBuf,
    pub package: String,
    pub artifact: PathBuf,
    pub target: PathBuf,
    pub sources: Vec<PathBuf>,
    pub features: String,
    pub snapshots: PathBuf,
}
impl ReloadConfig {
    pub fn from_environment(repo: &Path) -> Self {
        let path = |name, fallback: PathBuf| std::env::var_os(name).map_or(fallback, PathBuf::from);
        Self {
            manifest: path("BLOCKUI_DEV_MANIFEST", repo.join("Cargo.toml")),
            package: std::env::var("BLOCKUI_DEV_PACKAGE").unwrap_or_else(|_| "counter".into()),
            artifact: path(
                "BLOCKUI_DEV_ARTIFACT",
                repo.join("target/debug").join(format!(
                    "{}counter{}",
                    std::env::consts::DLL_PREFIX,
                    std::env::consts::DLL_SUFFIX
                )),
            ),
            target: path("BLOCKUI_DEV_TARGET", repo.join("target")),
            sources: std::env::var_os("BLOCKUI_DEV_SOURCES")
                .map(|paths| std::env::split_paths(&paths).collect())
                .unwrap_or_else(|| {
                    vec![
                        repo.join("crates/counter"),
                        repo.join("crates/blockui"),
                        repo.join("crates/blockui-abi"),
                        repo.join("Cargo.toml"),
                        repo.join("Cargo.lock"),
                    ]
                }),
            features: std::env::var("BLOCKUI_DEV_FEATURES").unwrap_or_default(),
            snapshots: repo.join(".runtime"),
        }
    }
    pub fn from_file(path: &Path, repo: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let string = |name: &str| {
            value[name]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("missing reload config {name}"))
        };
        Ok(Self {
            manifest: string("BLOCKUI_DEV_MANIFEST")?.into(),
            package: string("BLOCKUI_DEV_PACKAGE")?,
            artifact: string("BLOCKUI_DEV_ARTIFACT")?.into(),
            target: string("BLOCKUI_DEV_TARGET")?.into(),
            sources: std::env::split_paths(&string("BLOCKUI_DEV_SOURCES")?).collect(),
            features: string("BLOCKUI_DEV_FEATURES")?,
            snapshots: repo.join(".runtime"),
        })
    }
    #[cfg(not(feature = "hotreload"))]
    pub(super) fn refresh(&mut self, _stop: &AtomicBool) -> Result<(), String> {
        Err("native hot reload is disabled; enable the runtime hotreload feature".into())
    }
    #[cfg(feature = "hotreload")]
    pub(super) fn refresh(&mut self, stop: &AtomicBool) -> Result<(), String> {
        let mut command = blockui_dev::project::cargo_command();
        command
            .args(["metadata", "--format-version", "1", "--manifest-path"])
            .arg(&self.manifest);
        if !self.features.is_empty() {
            command.args(["--features", &self.features]);
        }
        command
            .arg("--filter-platform")
            .arg(blockui_dev::project::native_platform()?);
        let output = crate::dev_process::output(&mut command, stop, &self.snapshots)?;
        if !output.status.success() {
            return Err(crate::dev_process::failure("metadata", &output));
        }
        let value: serde_json::Value =
            serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
        let value = blockui_dev::configuration(
            &value,
            &self.manifest,
            &self.package,
            &self.target,
            &self.features,
        )?;
        let roots = value["BLOCKUI_DEV_SOURCES"]
            .as_str()
            .ok_or("missing dependency sources")?;
        let artifact = value["BLOCKUI_DEV_ARTIFACT"]
            .as_str()
            .ok_or("missing actor artifact")?;
        let artifact = Path::new(artifact)
            .file_name()
            .ok_or("invalid actor artifact")?;
        self.sources = std::env::split_paths(roots).collect();
        self.artifact = self.target.join("debug").join(artifact);
        Ok(())
    }
}
pub(super) fn source_snapshot(
    roots: &[PathBuf],
) -> BTreeMap<PathBuf, (std::time::SystemTime, u64)> {
    fn excluded(path: &Path) -> bool {
        path.file_name().is_some_and(|name| {
            [
                "target",
                ".git",
                ".runtime",
                "reports",
                "node_modules",
                "__pycache__",
            ]
            .iter()
            .any(|excluded| name == *excluded)
        })
    }
    fn visit(
        path: &Path,
        output: &mut BTreeMap<PathBuf, (std::time::SystemTime, u64)>,
        visited: &mut BTreeSet<PathBuf>,
        canonical_parent: bool,
    ) {
        let Ok(link_meta) = std::fs::symlink_metadata(path) else {
            return;
        };
        // Record the link itself, including dangling links, as well as target content.
        if link_meta.is_symlink()
            && let Ok(time) = link_meta.modified()
        {
            output.insert(path.into(), (time, link_meta.len()));
        }
        let resolved = if cfg!(unix) && canonical_parent && !link_meta.is_symlink() {
            // read_dir supplied the real entry name under an already canonical
            // parent. Only links and roots need another canonicalization.
            path.to_owned()
        } else {
            let Ok(resolved) = std::fs::canonicalize(path) else {
                return;
            };
            resolved
        };
        if excluded(&resolved) {
            return;
        }
        let meta = if cfg!(unix) && !link_meta.is_symlink() {
            link_meta
        } else {
            let Ok(meta) = std::fs::metadata(&resolved) else {
                return;
            };
            meta
        };
        // A snapshot stores canonical file identities, so aliases and overlapping
        // roots can share one directory traversal. Record each link above first.
        if meta.is_file() {
            if let Ok(time) = meta.modified() {
                output.insert(resolved, (time, meta.len()));
            }
        } else if meta.is_dir()
            && visited.insert(resolved.clone())
            && let Ok(entries) = std::fs::read_dir(&resolved)
        {
            for entry in entries.flatten() {
                if !excluded(&entry.path()) {
                    visit(&entry.path(), output, visited, true);
                }
            }
        }
    }
    let mut output = BTreeMap::new();
    let mut visited = BTreeSet::new();
    for root in roots {
        visit(root, &mut output, &mut visited, false);
    }
    output
}

pub(super) fn source_change_summary(
    previous: &BTreeMap<PathBuf, (std::time::SystemTime, u64)>,
    current: &BTreeMap<PathBuf, (std::time::SystemTime, u64)>,
    root: &Path,
) -> String {
    let changed: BTreeSet<_> = previous
        .keys()
        .chain(current.keys())
        .filter(|path| previous.get(*path) != current.get(*path))
        .collect();
    let Some(first) = changed.first() else {
        return "source graph refreshed".into();
    };
    let path = first
        .strip_prefix(root)
        .unwrap_or(first)
        .display()
        .to_string();
    let display: String = path.chars().take(160).collect();
    let suffix = if display.len() < path.len() {
        "…"
    } else {
        ""
    };
    format!("{} changed file(s): {display}{suffix}", changed.len())
}

#[cfg(feature = "hotreload")]
pub(super) fn build_actor(
    config: &ReloadConfig,
    stop: &AtomicBool,
    obsolete: impl FnMut() -> bool,
) -> Result<std::process::Output, String> {
    let mut command = blockui_dev::project::cargo_command();
    command
        .args([
            "rustc",
            "--quiet",
            "--lib",
            "--crate-type",
            "cdylib",
            "--package",
            &config.package,
            "--manifest-path",
        ])
        .arg(&config.manifest)
        .arg("--target-dir")
        .arg(&config.target);
    if !config.features.is_empty() {
        command.args(["--features", &config.features]);
    }
    crate::dev_process::output_until(&mut command, stop, &config.snapshots, obsolete)
}

#[cfg(feature = "hotreload")]
pub(super) fn watch(
    mut config: ReloadConfig,
    wake: Arc<Mutex<Option<shell::WakeCallback>>>,
) -> (Watch, crate::build_queue::Receiver) {
    static NEXT_WATCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    static BUILD_LOCK: Mutex<()> = Mutex::new(());
    let watch_id = NEXT_WATCH.fetch_add(1, Ordering::Relaxed);
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    let (sender, receiver) = crate::build_queue::channel();
    let mut previous = source_snapshot(&config.sources);
    let ready = Arc::new(AtomicBool::new(false));
    let worker_ready = ready.clone();
    let published = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let worker_published = published.clone();
    let superseded = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let worker_superseded = superseded.clone();
    let worker = thread::spawn(move || {
        let mut refresh_failed = match config.refresh(&stopped) {
            Ok(()) => false,
            Err(error) => {
                eprintln!("Cannot refresh watch roots: {error}");
                true
            }
        };
        let mut last_refresh = Instant::now();
        let mut last_error = String::new();
        for (path, value) in source_snapshot(&config.sources) {
            previous.entry(path).or_insert(value);
        }
        worker_ready.store(true, Ordering::Release);
        eprintln!("Watching {} for Rust changes", config.package);
        let mut generation = 0;
        let mut changed_sources = String::from("source graph refreshed");
        // Trade some idle filesystem scanning for faster first-save feedback.
        // Bursts retain the shorter interval for two seconds after each save.
        let mut active_until = Instant::now();
        'watch: while !stopped.load(Ordering::Relaxed) {
            thread::sleep(if Instant::now() < active_until {
                Duration::from_millis(10)
            } else {
                Duration::from_millis(25)
            });
            let current = source_snapshot(&config.sources);
            if current == previous
                && !(refresh_failed && last_refresh.elapsed() >= Duration::from_millis(500))
            {
                continue;
            }
            if current != previous {
                changed_sources = source_change_summary(
                    &previous,
                    &current,
                    config.manifest.parent().unwrap_or(Path::new(".")),
                );
                active_until = Instant::now() + Duration::from_secs(2);
                sender.clear(); // A newly observed save invalidates queued older code.
                last_error.clear();
            }
            let manifests_changed = current.iter().chain(previous.iter()).any(|(path, _)| {
                matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("Cargo.toml" | "Cargo.lock")
                ) && current.get(path) != previous.get(path)
            });
            previous = current;
            if manifests_changed || refresh_failed {
                last_refresh = Instant::now();
                if let Err(error) = config.refresh(&stopped) {
                    refresh_failed = true;
                    if error != last_error {
                        if sender.publish(BuildResult {
                            artifact: None,
                            message: error.clone(),
                        }) {
                            last_error = error;
                        }
                        if let Some(wake) = wake.lock().unwrap().as_ref() {
                            wake();
                        }
                    }
                    continue;
                }
                refresh_failed = false;
                last_error.clear();
                previous = source_snapshot(&config.sources);
            }
            // Hold through the snapshot copy: Cargo releases its own lock before
            // returning, allowing a different feature build to overwrite this artifact.
            let build_lock = loop {
                if stopped.load(Ordering::Relaxed) {
                    break 'watch;
                }
                match BUILD_LOCK.try_lock() {
                    Ok(lock) => break lock,
                    Err(std::sync::TryLockError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(20))
                    }
                    Err(std::sync::TryLockError::Poisoned(error)) => break error.into_inner(),
                }
            };
            if source_snapshot(&config.sources) != previous {
                drop(build_lock);
                continue;
            }
            let started = Instant::now();
            let mut checked = Instant::now();
            let output = build_actor(&config, &stopped, || {
                if checked.elapsed() < Duration::from_millis(20) {
                    return false;
                }
                checked = Instant::now();
                source_snapshot(&config.sources) != previous
            });
            if output
                .as_ref()
                .is_err_and(|error| error == "development build superseded")
            {
                worker_superseded.fetch_add(1, Ordering::Release);
                active_until = Instant::now() + Duration::from_secs(2);
                sender.clear();
                drop(build_lock);
                // Keep the old source stamp so the next iteration builds the latest save.
                continue;
            }
            let result = match output {
                Ok(output) if output.status.success() => {
                    generation += 1;
                    let snapshot = config.snapshots.join(format!(
                        "pending-{}-{watch_id}-{generation}{}",
                        std::process::id(),
                        std::env::consts::DLL_SUFFIX
                    ));
                    match crate::artifact::copy(&config.artifact, &snapshot) {
                        Ok(_) => BuildResult {
                            artifact: Some(snapshot),
                            message: format!(
                                "Built in {:.0} ms; {changed_sources}",
                                started.elapsed().as_secs_f64() * 1000.0
                            ),
                        },
                        Err(error) => BuildResult {
                            artifact: None,
                            message: error.to_string(),
                        },
                    }
                }
                Ok(output) => BuildResult {
                    artifact: None,
                    message: crate::dev_process::failure("build", &output),
                },
                Err(error) => BuildResult {
                    artifact: None,
                    message: error.to_string(),
                },
            };
            drop(build_lock);
            if source_snapshot(&config.sources) != previous {
                if let Some(path) = result.artifact {
                    let _ = std::fs::remove_file(path);
                }
                // Leave `previous` unchanged so the next iteration builds the new save.
                continue;
            }
            if !sender.publish(result) {
                break;
            }
            worker_published.fetch_add(1, Ordering::Release);
            if let Some(wake) = wake.lock().unwrap().as_ref() {
                wake();
            }
        }
    });
    (
        Watch {
            stop,
            thread: Some(worker),
            ready,
            published,
            superseded,
        },
        crate::build_queue::Receiver::Channel(receiver),
    )
}

#[cfg(not(feature = "hotreload"))]
pub(super) fn watch(
    _config: ReloadConfig,
    _wake: Arc<Mutex<Option<shell::WakeCallback>>>,
) -> (Watch, crate::build_queue::Receiver) {
    let (sender, receiver) = mpsc::channel();
    let _ = sender.send(BuildResult {
        artifact: None,
        message: "native hot reload is disabled".into(),
    });
    (
        Watch {
            stop: Arc::new(AtomicBool::new(false)),
            thread: None,
            ready: Arc::new(AtomicBool::new(true)),
            published: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            superseded: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        },
        crate::build_queue::Receiver::Channel(receiver),
    )
}
