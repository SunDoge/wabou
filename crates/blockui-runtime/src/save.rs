//! Host-only complete-file replacement. No plugin code or borrowed data survives.
use std::{
    ffi::CString,
    fs::{File, OpenOptions, Permissions},
    io::Write,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::OsStrExt,
            fs::{OpenOptionsExt, PermissionsExt},
        },
    },
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(1);
struct Temporary<'a> {
    directory: &'a File,
    name: CString,
}
impl Drop for Temporary<'_> {
    fn drop(&mut self) {
        unsafe {
            libc::unlinkat(self.directory.as_raw_fd(), self.name.as_ptr(), 0);
        }
    }
}
#[cfg(test)]
fn replace(path: &Path, bytes: &[u8]) -> Result<(), String> {
    replace_with(path, bytes, || Ok(()))
}
#[cfg(test)]
fn create(path: &Path, bytes: &[u8]) -> Result<(), String> {
    publish_with(path, bytes, true, || Ok(()))
}
/// A host-owned cancellation check runs immediately before irreversible publication.
/// Cancellation racing after that check may still publish; this is not rollback.
pub(super) fn publish_cancellable(
    path: &Path,
    bytes: &[u8],
    create: bool,
    cancelled: impl FnOnce() -> bool,
) -> Result<(), String> {
    publish_with(path, bytes, create, || {
        if cancelled() {
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "save cancelled before publication",
            ))
        } else {
            Ok(())
        }
    })
}
#[cfg(test)]
fn replace_with(
    path: &Path,
    bytes: &[u8],
    before_rename: impl FnOnce() -> std::io::Result<()>,
) -> Result<(), String> {
    publish_with(path, bytes, false, before_rename)
}
fn publish_with(
    path: &Path,
    bytes: &[u8],
    create: bool,
    before_publish: impl FnOnce() -> std::io::Result<()>,
) -> Result<(), String> {
    let parent = path.parent().ok_or("save target has no directory")?;
    let name = CString::new(
        path.file_name()
            .ok_or("save target has no filename")?
            .as_bytes(),
    )
    .map_err(|_| "save filename contains NUL")?;
    let directory = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NONBLOCK | libc::O_NOCTTY)
        .open(parent)
        .map_err(|e| e.to_string())?;
    let permissions = if create {
        None
    } else {
        // Resolve the target relative to the captured directory, and reject
        // symlinks/devices without blocking the worker on a FIFO.
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        let result = unsafe {
            libc::fstatat(
                directory.as_raw_fd(),
                name.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let metadata = unsafe { metadata.assume_init() };
        if metadata.st_mode & libc::S_IFMT != libc::S_IFREG {
            return Err("save target must be an existing regular file".into());
        }
        // mode_t is narrower than u32 on some Unix platforms.
        #[allow(clippy::unnecessary_cast)]
        let mode = metadata.st_mode as u32;
        Some(Permissions::from_mode(mode))
    };
    let mut attempts = 0;
    let (mut file, temporary) = loop {
        attempts += 1;
        if attempts > 128 {
            return Err("save temporary name collisions exceeded limit".into());
        }
        let id = NEXT
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| "save temporary identity exhausted")?;
        let candidate = CString::new(format!(".identity-save-{}-{id}", std::process::id()))
            .expect("generated temporary filename contains no NUL");
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                candidate.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC,
                0o600,
            )
        };
        if fd >= 0 {
            break (
                unsafe { File::from_raw_fd(fd) },
                Temporary {
                    directory: &directory,
                    name: candidate,
                },
            );
        }
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(error.to_string());
        }
    };
    file.write_all(bytes).map_err(|e| e.to_string())?;
    if let Some(permissions) = permissions {
        file.set_permissions(permissions)
            .map_err(|e| e.to_string())?;
    }
    file.sync_all().map_err(|e| e.to_string())?;
    before_publish().map_err(|e| e.to_string())?;
    if create {
        // Hard-link publication is atomic and fails if any destination already exists.
        // Staging is in the same directory, so this cannot cross filesystems.
        let result = unsafe {
            libc::linkat(
                directory.as_raw_fd(),
                temporary.name.as_ptr(),
                directory.as_raw_fd(),
                name.as_ptr(),
                0,
            )
        };
        if result != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let result = unsafe { libc::unlinkat(directory.as_raw_fd(), temporary.name.as_ptr(), 0) };
        if result != 0 {
            return Err(format!(
                "file created but staging cleanup failed: {}",
                std::io::Error::last_os_error()
            ));
        }
    } else {
        let result = unsafe {
            libc::renameat(
                directory.as_raw_fd(),
                temporary.name.as_ptr(),
                directory.as_raw_fd(),
                name.as_ptr(),
            )
        };
        if result != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    directory
        .sync_all()
        .map_err(|e| format!("file published but directory sync failed: {e}"))?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    #[test]
    fn cancellation_at_publication_preserves_target_and_cleans_staging() {
        for create in [false, true] {
            let root = std::env::temp_dir().join(format!(
                "identity-save-cancel-{}-{create}",
                std::process::id()
            ));
            fs::create_dir(&root).unwrap();
            let path = root.join("document");
            if !create {
                fs::write(&path, b"original").unwrap();
            }
            let mut checked = false;
            let result = publish_cancellable(&path, b"new contents", create, || {
                checked = true;
                // Check occurs after a complete synced staging file exists.
                assert_eq!(
                    fs::read_dir(&root).unwrap().count(),
                    if create { 1 } else { 2 }
                );
                true
            });
            assert!(checked);
            assert!(result.unwrap_err().contains("cancelled before publication"));
            if create {
                assert!(!path.exists());
                assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
            } else {
                assert_eq!(fs::read(&path).unwrap(), b"original");
                assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
            }
            publish_cancellable(&path, b"accepted", create, || false).unwrap();
            assert_eq!(fs::read(&path).unwrap(), b"accepted");
            assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn publication_stays_in_open_directory_when_parent_is_replaced() {
        for create in [false, true] {
            let root = std::env::temp_dir().join(format!(
                "identity-save-directory-{}-{create}",
                std::process::id()
            ));
            fs::create_dir(&root).unwrap();
            let parent = root.join("current");
            let moved = root.join("moved");
            fs::create_dir(&parent).unwrap();
            let path = parent.join("document");
            if !create {
                fs::write(&path, b"old").unwrap();
            }
            publish_with(&path, b"saved", create, || {
                fs::rename(&parent, &moved)?;
                fs::create_dir(&parent)?;
                fs::write(parent.join("document"), b"unrelated")
            })
            .unwrap();
            assert_eq!(fs::read(moved.join("document")).unwrap(), b"saved");
            assert_eq!(fs::read(&path).unwrap(), b"unrelated");
            assert_eq!(fs::read_dir(&moved).unwrap().count(), 1);
            assert_eq!(fs::read_dir(&parent).unwrap().count(), 1);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn failed_publication_cleans_staging_in_renamed_directory() {
        let root = std::env::temp_dir().join(format!(
            "identity-save-directory-failure-{}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let parent = root.join("current");
        let moved = root.join("moved");
        fs::create_dir(&parent).unwrap();
        let path = parent.join("document");
        fs::write(&path, b"old").unwrap();
        assert!(
            replace_with(&path, b"new", || {
                fs::rename(&parent, &moved)?;
                Err(std::io::Error::other(
                    "injected failure after directory rename",
                ))
            })
            .is_err()
        );
        assert_eq!(fs::read(moved.join("document")).unwrap(), b"old");
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn new_file_publication_is_complete_private_and_never_overwrites_racing_targets() {
        let root =
            std::env::temp_dir().join(format!("identity-create-file-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let path = root.join("document");
        create(&path, "新文档😀".as_bytes()).unwrap();
        assert_eq!(fs::read(&path).unwrap(), "新文档😀".as_bytes());
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o077, 0);
        assert!(create(&path, b"overwrite").is_err());
        assert_eq!(fs::read(&path).unwrap(), "新文档😀".as_bytes());
        let race = root.join("race");
        assert!(publish_with(&race, b"ours", true, || fs::write(&race, b"other writer")).is_err());
        assert_eq!(fs::read(&race).unwrap(), b"other writer");
        let failed = root.join("failed");
        assert!(
            publish_with(&failed, b"partial", true, || Err(std::io::Error::other(
                "stop before publication"
            )))
            .is_err()
        );
        assert!(!failed.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn replacement_preserves_mode_and_old_inode_and_cleans_failed_staging() {
        let root = std::env::temp_dir().join(format!("identity-save-proof-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let path = root.join("document");
        fs::write(&path, b"original long contents").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        let original = File::open(&path).unwrap();
        assert!(
            replace_with(&path, b"broken", || Err(std::io::Error::other(
                "injected pre-rename failure"
            )))
            .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), b"original long contents");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        let backup = root.join("backup");
        let result = replace_with(&path, b"failed rename", || {
            fs::rename(&path, &backup)?;
            fs::create_dir(&path)
        });
        assert!(result.is_err());
        assert!(path.is_dir());
        assert_eq!(fs::read(&backup).unwrap(), b"original long contents");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2); // no leaked staging file
        fs::remove_dir(&path).unwrap();
        fs::rename(&backup, &path).unwrap();
        replace(&path, b"saved").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"saved");
        assert_ne!(
            original.metadata().unwrap().ino(),
            fs::metadata(&path).unwrap().ino()
        );
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}
