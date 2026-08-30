use std::path::{Path, PathBuf};

use gix::{bstr::ByteSlice as _, object::tree::EntryKind, refs::transaction::PreviousValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CaptureCheckpointRequest {
    pub(crate) cwd: PathBuf,
    pub(crate) namespace: String,
    pub(crate) sequence: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorktreeCheckpoint {
    pub(crate) commit_id: String,
    pub(crate) git_ref: String,
    pub(crate) skipped_paths: Vec<String>,
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
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
