use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use gix::{bstr::ByteSlice as _, object::tree::EntryKind, refs::transaction::PreviousValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CaptureCheckpointRequest {
    pub(crate) cwd: PathBuf,
    pub(crate) namespace: String,
    pub(crate) sequence: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RestoreCheckpointRequest {
    pub(crate) cwd: PathBuf,
    pub(crate) commit_id: String,
    pub(crate) namespace: String,
    pub(crate) sequence: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RetainCheckpointRequest {
    pub(crate) cwd: PathBuf,
    pub(crate) commit_id: String,
    pub(crate) session_id: String,
    pub(crate) entry_id: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FindCheckpointRequest {
    pub(crate) cwd: PathBuf,
    pub(crate) session_id: String,
    pub(crate) entry_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorktreeCheckpoint {
    pub(crate) commit_id: String,
    pub(crate) git_ref: String,
    pub(crate) skipped_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorktreeRestore {
    pub(crate) safety_checkpoint: WorktreeCheckpoint,
    pub(crate) changed_paths: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TreeFile {
    kind: EntryKind,
    oid: gix::ObjectId,
}

pub(crate) fn capture_worktree(
    cwd: &Path,
    namespace: &str,
    sequence: u64,
) -> Result<WorktreeCheckpoint, String> {
    let repository = gix::open(cwd)
        .map_err(|error| format!("{} is not a Git repository: {error}", cwd.display()))?;
    let root = repository
        .workdir()
        .ok_or_else(|| "worktree checkpoints require a non-bare repository".to_owned())?;
    let base_tree = match repository.head_tree_id() {
        Ok(id) => repository
            .find_tree(id)
            .map_err(|error| format!("could not read HEAD tree: {error}"))?,
        Err(_) => repository.empty_tree(),
    };
    let mut editor = base_tree
        .edit()
        .map_err(|error| format!("could not prepare checkpoint tree: {error}"))?;
    let mut skipped_paths = Vec::new();

    for path in super::service::repository_changed_paths(&repository)? {
        let relative = repository_path(&path)?;
        let absolute = root.join(&path);
        let metadata = match std::fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                editor.remove(relative.as_str()).map_err(|error| {
                    format!("could not remove {relative} from checkpoint: {error}")
                })?;
                continue;
            }
            Err(error) => return Err(format!("could not inspect {}: {error}", absolute.display())),
        };

        let (kind, contents) = if metadata.file_type().is_symlink() {
            (EntryKind::Link, symlink_contents(&absolute)?)
        } else if metadata.is_file() {
            (
                file_kind(&metadata),
                std::fs::read(&absolute)
                    .map_err(|error| format!("could not read {}: {error}", absolute.display()))?,
            )
        } else {
            skipped_paths.push(relative);
            continue;
        };
        let blob = repository
            .write_blob(contents)
            .map_err(|error| format!("could not write checkpoint blob for {relative}: {error}"))?;
        editor
            .upsert(relative.as_str(), kind, blob.detach())
            .map_err(|error| format!("could not add {relative} to checkpoint: {error}"))?;
    }

    let tree = editor
        .write()
        .map_err(|error| format!("could not write checkpoint tree: {error}"))?;
    let parent = repository.head_id().ok().map(|id| id.detach());
    let timestamp = format!(
        "{} +0000",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    let signature = gix::actor::SignatureRef {
        name: b"Wabou Checkpoint".as_bstr(),
        email: b"checkpoint@wabou.local".as_bstr(),
        time: &timestamp,
    };
    let parents = parent.into_iter().collect::<Vec<_>>();
    let commit = repository
        .new_commit_as(
            signature,
            signature,
            format!("Wabou checkpoint {namespace}/{sequence}"),
            tree.detach(),
            parents,
        )
        .map_err(|error| format!("could not write checkpoint commit: {error}"))?;
    let git_ref = format!(
        "refs/wabou/pi-agent/{}/{}",
        sanitize_ref_component(namespace),
        sequence
    );
    repository
        .reference(
            git_ref.as_str(),
            commit.id,
            PreviousValue::Any,
            "wabou checkpoint",
        )
        .map_err(|error| format!("could not retain checkpoint reference: {error}"))?;

    Ok(WorktreeCheckpoint {
        commit_id: commit.id.to_string(),
        git_ref,
        skipped_paths,
    })
}

pub(crate) fn restore_worktree(
    cwd: &Path,
    commit_id: &str,
    namespace: &str,
    sequence: u64,
) -> Result<WorktreeRestore, String> {
    let safety_checkpoint = capture_worktree(cwd, &format!("{namespace}-safety"), sequence)?;
    let repository = gix::open(cwd)
        .map_err(|error| format!("{} is not a Git repository: {error}", cwd.display()))?;
    let root = repository
        .workdir()
        .ok_or_else(|| "worktree checkpoints require a non-bare repository".to_owned())?;
    let target = commit_tree(&repository, commit_id)?;
    let current = commit_tree(&repository, &safety_checkpoint.commit_id)?;
    let changed_paths = changed_tree_paths(&current, &target);

    if let Err(error) = apply_tree_changes(&repository, root, &current, &target, &changed_paths) {
        let rollback_paths = changed_tree_paths(&target, &current);
        let rollback = apply_tree_changes(&repository, root, &target, &current, &rollback_paths);
        return Err(match rollback {
            Ok(()) => format!("could not restore checkpoint; safety snapshot restored: {error}"),
            Err(rollback_error) => format!(
                "could not restore checkpoint ({error}); safety rollback also failed ({rollback_error})"
            ),
        });
    }

    Ok(WorktreeRestore {
        safety_checkpoint,
        changed_paths,
    })
}

pub(crate) fn retain_for_entry(
    cwd: &Path,
    commit_id: &str,
    session_id: &str,
    entry_id: &str,
) -> Result<WorktreeCheckpoint, String> {
    let repository = gix::open(cwd)
        .map_err(|error| format!("{} is not a Git repository: {error}", cwd.display()))?;
    let commit_id = gix::ObjectId::from_hex(commit_id.as_bytes())
        .map_err(|error| format!("invalid checkpoint commit id: {error}"))?;
    repository
        .find_commit(commit_id)
        .map_err(|error| format!("could not read checkpoint commit {commit_id}: {error}"))?;
    let git_ref = entry_ref(session_id, entry_id);
    repository
        .reference(
            git_ref.as_str(),
            commit_id,
            PreviousValue::Any,
            "wabou checkpoint entry",
        )
        .map_err(|error| format!("could not retain checkpoint entry reference: {error}"))?;
    Ok(WorktreeCheckpoint {
        commit_id: commit_id.to_string(),
        git_ref,
        skipped_paths: Vec::new(),
    })
}

pub(crate) fn find_for_entry(
    cwd: &Path,
    session_id: &str,
    entry_id: &str,
) -> Result<Option<WorktreeCheckpoint>, String> {
    let repository = gix::open(cwd)
        .map_err(|error| format!("{} is not a Git repository: {error}", cwd.display()))?;
    let git_ref = entry_ref(session_id, entry_id);
    let Some(reference) = repository
        .try_find_reference(git_ref.as_str())
        .map_err(|error| format!("could not find checkpoint entry reference: {error}"))?
    else {
        return Ok(None);
    };
    Ok(Some(WorktreeCheckpoint {
        commit_id: reference.id().to_string(),
        git_ref,
        skipped_paths: Vec::new(),
    }))
}

fn entry_ref(session_id: &str, entry_id: &str) -> String {
    format!(
        "refs/wabou/pi-agent/entries/{}/{}",
        encode_ref_component(session_id),
        encode_ref_component(entry_id)
    )
}

fn encode_ref_component(value: &str) -> String {
    if value.is_empty() {
        return "empty".to_owned();
    }
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn commit_tree(
    repository: &gix::Repository,
    commit_id: &str,
) -> Result<BTreeMap<String, TreeFile>, String> {
    let id = gix::ObjectId::from_hex(commit_id.as_bytes())
        .map_err(|error| format!("invalid checkpoint commit id: {error}"))?;
    let commit = repository
        .find_commit(id)
        .map_err(|error| format!("could not read checkpoint commit {commit_id}: {error}"))?;
    let tree = commit
        .tree()
        .map_err(|error| format!("could not read checkpoint tree {commit_id}: {error}"))?;
    tree.traverse()
        .breadthfirst
        .files()
        .map_err(|error| format!("could not traverse checkpoint tree: {error}"))?
        .into_iter()
        .map(|entry| {
            let path = entry
                .filepath
                .to_str()
                .map_err(|_| "checkpoint paths must be valid UTF-8".to_owned())?
                .to_owned();
            repository_path(Path::new(&path))?;
            Ok((
                path,
                TreeFile {
                    kind: entry.mode.kind(),
                    oid: entry.oid,
                },
            ))
        })
        .collect()
}

fn changed_tree_paths(
    current: &BTreeMap<String, TreeFile>,
    target: &BTreeMap<String, TreeFile>,
) -> Vec<String> {
    current
        .keys()
        .chain(target.keys())
        .filter(|path| current.get(*path) != target.get(*path))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn apply_tree_changes(
    repository: &gix::Repository,
    root: &Path,
    current: &BTreeMap<String, TreeFile>,
    target: &BTreeMap<String, TreeFile>,
    changed_paths: &[String],
) -> Result<(), String> {
    for path in changed_paths
        .iter()
        .filter(|path| !target.contains_key(*path))
    {
        remove_worktree_path(root, path)?;
    }
    for path in changed_paths {
        let Some(file) = target.get(path) else {
            continue;
        };
        if file.kind == EntryKind::Commit {
            return Err(format!("cannot restore submodule path `{path}`"));
        }
        let object = repository
            .find_object(file.oid)
            .map_err(|error| format!("could not read checkpoint object for {path}: {error}"))?;
        write_worktree_path(root, path, file.kind, &object.data)?;
    }
    for path in changed_paths
        .iter()
        .filter(|path| current.contains_key(*path))
    {
        remove_empty_parents(root, path);
    }
    Ok(())
}

fn remove_worktree_path(root: &Path, relative: &str) -> Result<(), String> {
    let path = root.join(relative);
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            std::fs::remove_dir_all(&path)
        }
        Ok(_) => std::fs::remove_file(&path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("could not inspect {}: {error}", path.display())),
    }
    .map_err(|error| format!("could not remove {}: {error}", path.display()))
}

fn write_worktree_path(
    root: &Path,
    relative: &str,
    kind: EntryKind,
    contents: &[u8],
) -> Result<(), String> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }
    if std::fs::symlink_metadata(&path).is_ok() {
        remove_worktree_path(root, relative)?;
    }
    if kind == EntryKind::Link {
        create_symlink(contents, &path)?;
    } else {
        std::fs::write(&path, contents)
            .map_err(|error| format!("could not write {}: {error}", path.display()))?;
        set_executable(&path, kind == EntryKind::BlobExecutable)?;
    }
    Ok(())
}

fn remove_empty_parents(root: &Path, relative: &str) {
    let mut parent = root.join(relative).parent().map(Path::to_path_buf);
    while let Some(path) = parent {
        if path == root || std::fs::remove_dir(&path).is_err() {
            break;
        }
        parent = path.parent().map(Path::to_path_buf);
    }
}

fn repository_path(path: &Path) -> Result<String, String> {
    let value = path
        .to_str()
        .ok_or_else(|| "checkpoint paths must be valid UTF-8".to_owned())?
        .replace('\\', "/");
    if value.is_empty() || value.starts_with('/') || value.split('/').any(|part| part == "..") {
        return Err(format!(
            "invalid repository-relative checkpoint path `{value}`"
        ));
    }
    Ok(value)
}

fn sanitize_ref_component(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    let sanitized = sanitized.trim_matches('-');
    if sanitized.is_empty() {
        "checkpoint".to_owned()
    } else {
        sanitized.to_owned()
    }
}

#[cfg(unix)]
fn file_kind(metadata: &std::fs::Metadata) -> EntryKind {
    use std::os::unix::fs::PermissionsExt as _;
    if metadata.permissions().mode() & 0o111 != 0 {
        EntryKind::BlobExecutable
    } else {
        EntryKind::Blob
    }
}

#[cfg(not(unix))]
fn file_kind(_metadata: &std::fs::Metadata) -> EntryKind {
    EntryKind::Blob
}

fn symlink_contents(path: &Path) -> Result<Vec<u8>, String> {
    let target = std::fs::read_link(path)
        .map_err(|error| format!("could not read symlink {}: {error}", path.display()))?;
    os_string_bytes(target)
}

#[cfg(unix)]
fn os_string_bytes(path: PathBuf) -> Result<Vec<u8>, String> {
    use std::os::unix::ffi::OsStrExt as _;
    Ok(path.as_os_str().as_bytes().to_vec())
}

#[cfg(unix)]
fn create_symlink(target: &[u8], path: &Path) -> Result<(), String> {
    use std::os::{unix::ffi::OsStrExt as _, unix::fs::symlink};
    symlink(std::ffi::OsStr::from_bytes(target), path)
        .map_err(|error| format!("could not create symlink {}: {error}", path.display()))
}

#[cfg(windows)]
fn create_symlink(target: &[u8], path: &Path) -> Result<(), String> {
    let target = std::str::from_utf8(target)
        .map_err(|_| "symlink targets must be valid UTF-8 on Windows".to_owned())?;
    std::os::windows::fs::symlink_file(target, path)
        .map_err(|error| format!("could not create symlink {}: {error}", path.display()))
}

#[cfg(not(any(unix, windows)))]
fn create_symlink(_target: &[u8], path: &Path) -> Result<(), String> {
    Err(format!(
        "restoring symlinks is not supported on this platform: {}",
        path.display()
    ))
}

#[cfg(unix)]
fn set_executable(path: &Path, executable: bool) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt as _;
    let mut permissions = std::fs::metadata(path)
        .map_err(|error| format!("could not inspect {}: {error}", path.display()))?
        .permissions();
    let mut mode = permissions.mode();
    if executable {
        mode |= 0o111;
    } else {
        mode &= !0o111;
    }
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions)
        .map_err(|error| format!("could not set permissions for {}: {error}", path.display()))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path, _executable: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(not(unix))]
fn os_string_bytes(path: PathBuf) -> Result<Vec<u8>, String> {
    path.to_str()
        .map(|value| value.as_bytes().to_vec())
        .ok_or_else(|| "symlink target must be valid UTF-8 on this platform".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "wabou-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).expect("temporary directory");
        path
    }

    fn signature() -> gix::actor::SignatureRef<'static> {
        gix::actor::SignatureRef {
            name: b"Wabou Test".as_bstr(),
            email: b"test@wabou.local".as_bstr(),
            time: "0 +0000",
        }
    }

    #[test]
    fn captures_modified_untracked_and_deleted_files_without_moving_head() {
        let root = temporary_directory("checkpoint-capture");
        let repository = gix::init(&root).expect("repository");
        let mut tree = repository.empty_tree().edit().expect("tree editor");
        let base = repository.write_blob(b"base\n").expect("base blob");
        let removed = repository.write_blob(b"remove me\n").expect("removed blob");
        tree.upsert("tracked.txt", EntryKind::Blob, base.detach())
            .expect("tracked entry");
        tree.upsert("removed.txt", EntryKind::Blob, removed.detach())
            .expect("removed entry");
        let tree_id = tree.write().expect("base tree");
        let base_tree_id = tree_id.detach();
        let commit = repository
            .commit_as(
                signature(),
                signature(),
                "HEAD",
                "base",
                base_tree_id,
                std::iter::empty::<gix::ObjectId>(),
            )
            .expect("base commit");
        repository
            .index_from_tree(&base_tree_id)
            .expect("base index")
            .write(Default::default())
            .expect("persisted index");
        std::fs::write(root.join("tracked.txt"), "changed\n").expect("tracked worktree");
        std::fs::write(root.join("untracked.txt"), "new\n").expect("untracked worktree");
        let index_before = std::fs::read(repository.index_path()).ok();

        let checkpoint = capture_worktree(&root, "session:one", 7).expect("checkpoint");

        assert_eq!(
            repository.head_id().expect("HEAD").detach(),
            commit.detach()
        );
        assert_eq!(std::fs::read(repository.index_path()).ok(), index_before);
        assert_eq!(checkpoint.git_ref, "refs/wabou/pi-agent/session-one/7");
        assert!(checkpoint.skipped_paths.is_empty());
        let retained = retain_for_entry(&root, &checkpoint.commit_id, "session/one", "entry:7")
            .expect("retained entry checkpoint");
        assert_eq!(
            find_for_entry(&root, "session/one", "entry:7").expect("checkpoint lookup"),
            Some(retained)
        );
        assert_eq!(
            find_for_entry(&root, "session/one", "missing").expect("missing checkpoint lookup"),
            None
        );
        let checkpoint_commit = repository
            .find_commit(
                gix::ObjectId::from_hex(checkpoint.commit_id.as_bytes()).expect("checkpoint id"),
            )
            .expect("checkpoint commit");
        let checkpoint_tree = checkpoint_commit.tree().expect("checkpoint tree");
        for (path, expected) in [
            ("tracked.txt", b"changed\n".as_slice()),
            ("untracked.txt", b"new\n".as_slice()),
        ] {
            let entry = checkpoint_tree
                .lookup_entry_by_path(path)
                .expect("tree lookup")
                .expect("captured entry");
            assert_eq!(entry.object().expect("blob").data, expected);
        }
        assert!(
            checkpoint_tree
                .lookup_entry_by_path("removed.txt")
                .expect("removed lookup")
                .is_none()
        );

        std::fs::write(root.join("tracked.txt"), "after checkpoint\n").expect("later edit");
        std::fs::remove_file(root.join("untracked.txt")).expect("later removal");
        std::fs::write(root.join("later.txt"), "later\n").expect("later untracked file");
        let restore = restore_worktree(&root, &checkpoint.commit_id, "session:one", 8)
            .expect("restored checkpoint");

        assert_eq!(
            std::fs::read_to_string(root.join("tracked.txt")).expect("restored tracked file"),
            "changed\n"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("untracked.txt")).expect("restored untracked file"),
            "new\n"
        );
        assert!(!root.join("later.txt").exists());
        assert_eq!(
            repository.head_id().expect("HEAD after restore").detach(),
            commit.detach()
        );
        assert_eq!(std::fs::read(repository.index_path()).ok(), index_before);
        assert_eq!(
            restore.safety_checkpoint.git_ref,
            "refs/wabou/pi-agent/session-one-safety/8"
        );
        assert_eq!(
            restore.changed_paths,
            ["later.txt", "tracked.txt", "untracked.txt"]
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
