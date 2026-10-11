//! Development-only watcher and immutable, bounded browser generations.
use crate::web;
use blockui_dev::project::Project;
use serde_json::{Value, json};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    io::Write,
    net::TcpStream,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct Dev {
    pub status: Mutex<Value>,
    pub generations: PathBuf,
    pub clients: std::sync::atomic::AtomicUsize,
}
fn stamp(paths: &[PathBuf]) -> u64 {
    fn visit(path: &Path, hash: &mut DefaultHasher) {
        if path.is_symlink() {
            return;
        }
        if path.is_dir() {
            if matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some("target" | "pkg" | "node_modules" | ".git" | ".runtime" | "reports")
            ) {
                return;
            }
            if let Ok(entries) = fs::read_dir(path) {
                let mut entries: Vec<_> =
                    entries.filter_map(Result::ok).map(|e| e.path()).collect();
                entries.sort();
                for entry in entries {
                    visit(&entry, hash);
                }
            }
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("rs" | "toml" | "lock" | "js" | "css" | "html")
        ) {
            path.hash(hash);
            if let Ok(bytes) = fs::read(path) {
                bytes.hash(hash);
            }
        }
    }
    let mut hash = DefaultHasher::new();
    for path in paths {
        visit(path, &mut hash);
    }
    hash.finish()
}
fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_symlink() {
            return Err("generated assets cannot contain symlinks".into());
        }
        if path.is_dir() {
            copy_tree(&path, &destination.join(entry.file_name()))?;
        } else {
            fs::copy(path, destination.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
impl Dev {
    fn publish(&self, assets: &Path, generation: u64) -> Result<(), String> {
        let destination = self.generations.join(generation.to_string());
        if destination.exists() {
            fs::remove_dir_all(&destination).map_err(|e| e.to_string())?;
        }
        copy_tree(&assets.join("pkg"), &destination)?;
        let module = fs::read_dir(&destination)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .find_map(|e| {
                e.file_name()
                    .to_str()
                    .and_then(|n| n.strip_suffix("_bg.wasm"))
                    .map(str::to_owned)
            })
            .ok_or("missing Wasm module")?;
        let renderer_bytes = fs::read(destination.join("renderer.js"))
            .or_else(|_| fs::read(assets.join("renderer.js")))
            .map_err(|e| format!("cannot read browser renderer: {e}"))?;
        let mut renderer_hash = DefaultHasher::new();
        renderer_bytes.hash(&mut renderer_hash);
        let renderer = renderer_hash.finish();
        let mut status = self.status.lock().unwrap();
        let revision = status["revision"].as_u64().unwrap_or(0) + 1;
        *status = json!({"generation":generation,"revision":revision,"module":module,"renderer":renderer.to_string(),"error":null,"building":false});
        drop(status);
        for entry in fs::read_dir(&self.generations).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry
                .file_name()
                .to_str()
                .and_then(|n| n.parse::<u64>().ok())
                .is_some_and(|n| n + 1 < generation)
            {
                fs::remove_dir_all(entry.path()).map_err(|e| e.to_string())?;
            }
        }
        println!("Web generation {generation} ready (two snapshot generations retained)");
        Ok(())
    }
    fn message(&self, building: bool, error: Option<String>) {
        let mut status = self.status.lock().unwrap();
        status["revision"] = json!(status["revision"].as_u64().unwrap_or(0) + 1);
        status["building"] = json!(building);
        status["error"] = json!(error);
    }
    pub fn start(project: Project, features: String, assets: &Path) -> Result<Arc<Self>, String> {
        let runtime = project.package_root().join(".runtime");
        fs::create_dir_all(&runtime).map_err(|e| e.to_string())?;
        let lease = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(runtime.join("web-dev.lock"))
            .map_err(|e| e.to_string())?;
        lease
            .try_lock()
            .map_err(|_| "another Web dev server owns this project".to_string())?;
        let directory = runtime.join("web-generations");
        if directory.is_symlink() {
            return Err("Web generation directory cannot be a symlink".into());
        }
        if directory.exists() {
            fs::remove_dir_all(&directory).map_err(|e| e.to_string())?;
        }
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        let dev = Arc::new(Self {
            status: Mutex::new(json!({"revision":0})),
            generations: directory,
            clients: Default::default(),
        });
        dev.publish(assets, 1)?;
        let config = blockui_dev::configuration(
            &project.metadata,
            &project.manifest,
            project.name(),
            &project.target,
            &features,
        )?;
        let mut paths: Vec<_> = std::env::split_paths(
            config["BLOCKUI_DEV_SOURCES"]
                .as_str()
                .ok_or("missing watch sources")?,
        )
        .collect();
        paths.push(assets.to_owned());
        paths.push(project.sdk_root()?.join("crates/blockui-cli/assets"));
        let assets = assets.to_owned();
        let worker = dev.clone();
        let mut previous = stamp(&paths);
        std::thread::spawn(move || {
            let _lease = lease;
            let mut generation = 1;
            loop {
                std::thread::sleep(Duration::from_millis(250));
                let current = stamp(&paths);
                if current == previous {
                    continue;
                }
                std::thread::sleep(Duration::from_millis(150));
                previous = stamp(&paths);
                worker.message(true, None);
                match web::build(&project, &features)
                    .and_then(|_| worker.publish(&assets, generation + 1))
                {
                    Ok(()) => generation += 1,
                    Err(error) => {
                        eprintln!("Web build failed: {error}");
                        worker.message(false, Some(error));
                    }
                }
            }
        });
        Ok(dev)
    }
    pub fn events(&self, mut stream: TcpStream) -> std::io::Result<()> {
        use std::sync::atomic::Ordering;
        if self.clients.fetch_add(1, Ordering::Relaxed) >= 16 {
            self.clients.fetch_sub(1, Ordering::Relaxed);
            return stream.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        }
        let result = (|| {
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n"
            )?;
            let mut revision = None;
            loop {
                let status = self.status.lock().unwrap().clone();
                let next = status["revision"].as_u64();
                if revision != next {
                    writeln!(stream, "data: {status}\n")?;
                    revision = next;
                } else {
                    writeln!(stream, ": heartbeat\n")?;
                }
                stream.flush()?;
                std::thread::sleep(Duration::from_millis(500));
            }
        })();
        self.clients.fetch_sub(1, Ordering::Relaxed);
        result
    }
}
