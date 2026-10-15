use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use wabou::specta;
use wabou::{CapabilityContract, JsonMethod, NativeCapability, rquickjs};

pub const CAPABILITY: CapabilityContract = CapabilityContract::new("launcher", 1);
const LIST: JsonMethod<(), Vec<LauncherEntry>> = JsonMethod::no_request("list");
const LAUNCH: JsonMethod<LaunchRequest, bool> = JsonMethod::new("launch");

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bindings", derive(wabou::Type))]
#[serde(rename_all = "camelCase")]
pub struct LauncherEntry {
    id: String,
    label: String,
    description: String,
    keywords: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "bindings", derive(wabou::Type))]
#[serde(rename_all = "camelCase")]
struct LaunchRequest {
    id: String,
}

#[derive(Clone)]
pub struct LauncherService {
    entries: Arc<Vec<LauncherEntry>>,
    targets: Arc<HashMap<String, LaunchTarget>>,
}

#[derive(Clone)]
enum LaunchTarget {
    #[cfg(target_os = "linux")]
    DesktopFile(PathBuf),
    #[cfg(target_os = "macos")]
    Application(PathBuf),
}

impl LauncherService {
    pub fn discover() -> Self {
        let discovered = discover_entries();
        let entries = discovered.iter().map(|(entry, _)| entry.clone()).collect();
        let targets = discovered
            .into_iter()
            .map(|(entry, target)| (entry.id, target))
            .collect();
        Self {
            entries: Arc::new(entries),
            targets: Arc::new(targets),
        }
    }

    fn launch(&self, id: &str) -> Result<bool, String> {
        let target = self
            .targets
            .get(id)
            .ok_or_else(|| format!("application `{id}` is no longer available"))?;
        match target {
            #[cfg(target_os = "linux")]
            LaunchTarget::DesktopFile(path) => Command::new("gio")
                .arg("launch")
                .arg(path)
                .spawn()
                .map(|_| true)
                .map_err(|error| format!("failed to launch {}: {error}", path.display())),
            #[cfg(target_os = "macos")]
            LaunchTarget::Application(path) => Command::new("open")
                .arg(path)
                .spawn()
                .map(|_| true)
                .map_err(|error| format!("failed to launch {}: {error}", path.display())),
        }
    }
}

pub fn mount<'js>(
    capability: &mut NativeCapability<'js>,
    service: LauncherService,
) -> rquickjs::Result<()> {
    let list = service.entries.clone();
    capability.json_method(LIST, move |(): ()| {
        let entries = list.clone();
        async move { Ok::<_, String>((*entries).clone()) }
    })?;

    capability.json_method(LAUNCH, move |request: LaunchRequest| {
        let service = service.clone();
        async move { service.launch(&request.id) }
    })
}

#[cfg(target_os = "linux")]
fn discover_entries() -> Vec<(LauncherEntry, LaunchTarget)> {
    let mut roots = Vec::new();
    if let Some(home) = std::env::var_os("XDG_DATA_HOME") {
        roots.push(PathBuf::from(home).join("applications"));
    } else if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join(".local/share/applications"));
    }
    for root in std::env::var_os("XDG_DATA_DIRS")
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into())
        .to_string_lossy()
        .split(':')
        .filter(|root| !root.is_empty())
    {
        roots.push(PathBuf::from(root).join("applications"));
    }

    let mut paths = Vec::new();
    for root in roots {
        collect_desktop_files(&root, &mut paths);
    }
    paths.sort();

    let mut seen_ids = HashSet::new();
    let mut seen_labels = HashSet::new();
    let mut entries = Vec::new();
    for path in paths {
        let Some(parsed) = parse_desktop_entry(&path) else {
            continue;
        };
        if !seen_ids.insert(parsed.id.clone()) || !seen_labels.insert(parsed.label.to_lowercase()) {
            continue;
        }
        entries.push((parsed, LaunchTarget::DesktopFile(path)));
    }
    entries.sort_by(|left, right| {
        left.0
            .label
            .to_lowercase()
            .cmp(&right.0.label.to_lowercase())
    });
    entries
}

#[cfg(target_os = "linux")]
fn collect_desktop_files(root: &Path, output: &mut Vec<PathBuf>) {
    let Ok(children) = fs::read_dir(root) else {
        return;
    };
    for child in children.flatten() {
        let path = child.path();
        if path.is_dir() {
            collect_desktop_files(&path, output);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "desktop")
        {
            output.push(path);
        }
    }
}

#[cfg(target_os = "linux")]
fn parse_desktop_entry(path: &Path) -> Option<LauncherEntry> {
    let source = fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let mut values = HashMap::<&str, &str>::new();
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if matches!(
            key,
            "Name" | "GenericName" | "Comment" | "Keywords" | "Type" | "Hidden" | "NoDisplay"
        ) {
            values.entry(key).or_insert(value.trim());
        }
    }
    if values.get("Type").copied().unwrap_or("Application") != "Application"
        || matches!(values.get("Hidden"), Some(&"true"))
        || matches!(values.get("NoDisplay"), Some(&"true"))
    {
        return None;
    }
    let label = values.get("Name")?.trim();
    if label.is_empty() {
        return None;
    }
    let id = path.file_name()?.to_string_lossy().into_owned();
    let description = values
        .get("GenericName")
        .or_else(|| values.get("Comment"))
        .copied()
        .unwrap_or("Application")
        .trim()
        .to_owned();
    let mut keywords = values
        .get("Keywords")
        .copied()
        .unwrap_or_default()
        .split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    keywords.push(description.clone());
    Some(LauncherEntry {
        id,
        label: label.to_owned(),
        description,
        keywords,
    })
}

#[cfg(target_os = "macos")]
fn discover_entries() -> Vec<(LauncherEntry, LaunchTarget)> {
    let mut roots = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join("Applications"));
    }
    let mut paths = Vec::new();
    for root in roots {
        let Ok(children) = fs::read_dir(root) else {
            continue;
        };
        paths.extend(
            children
                .flatten()
                .map(|child| child.path())
                .filter(|path| path.extension().is_some_and(|extension| extension == "app")),
        );
    }
    paths.sort();
    paths
        .into_iter()
        .filter_map(|path| {
            let label = path.file_stem()?.to_string_lossy().into_owned();
            let id = path.file_name()?.to_string_lossy().into_owned();
            Some((
                LauncherEntry {
                    id,
                    label,
                    description: "Application".into(),
                    keywords: Vec::new(),
                },
                LaunchTarget::Application(path),
            ))
        })
        .collect()
}

/// Build the Rust-owned TypeScript client contract for this capability.
#[cfg(feature = "bindings")]
pub fn manifest() -> wabou::Bindings {
    wabou::Bindings::new().capability(
        wabou::Capability::new(CAPABILITY)
            .method(LIST)
            .method(LAUNCH),
    )
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn discover_entries() -> Vec<(LauncherEntry, LaunchTarget)> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn desktop_entry_parser_rejects_hidden_entries_and_keeps_search_metadata() {
        let directory = tempfile::tempdir().unwrap();
        let visible = directory.path().join("editor.desktop");
        fs::write(
            &visible,
            "[Desktop Entry]\nType=Application\nName=Paper\nGenericName=Text Editor\nKeywords=write;notes;\n",
        )
        .unwrap();
        let parsed = parse_desktop_entry(&visible).unwrap();
        assert_eq!(parsed.id, "editor.desktop");
        assert_eq!(parsed.label, "Paper");
        assert_eq!(parsed.description, "Text Editor");
        assert!(parsed.keywords.iter().any(|keyword| keyword == "notes"));

        let hidden = directory.path().join("hidden.desktop");
        fs::write(
            &hidden,
            "[Desktop Entry]\nType=Application\nName=Hidden\nNoDisplay=true\n",
        )
        .unwrap();
        assert!(parse_desktop_entry(&hidden).is_none());
    }

    #[cfg(feature = "bindings")]
    #[test]
    fn generated_host_bindings_are_current() {
        manifest()
            .check(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/ui/generated/host-bindings.ts"
            ))
            .expect("run `bun run gen` to refresh Launcher host bindings");
    }
}
