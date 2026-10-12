//! Host-owned regular file handles. No plugin pointer or destructor is retained.
use super::{Context, Turn};
#[cfg(target_arch = "wasm32")]
use crate::web_files::FileExt;
#[cfg(windows)]
use crate::windows_files::FileExt;
use blockui_abi::{Bytes, ERROR, OK};
#[cfg(unix)]
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::{
    collections::BTreeMap,
    ffi::c_void,
    fs::{File, OpenOptions},
    path::PathBuf,
    sync::Arc,
};
#[derive(Clone)]
pub(super) struct HostFile {
    pub file: Arc<File>,
    pub(super) path: PathBuf,
    policy: u32,
    pub(super) access: u32,
}
fn open_regular(path: &std::path::Path, access: u32) -> Result<File, String> {
    // Inspect the opened descriptor, never a path checked before open. A rename
    // to a FIFO must not leave the actor/UI thread waiting for a writer.
    let mut options = OpenOptions::new();
    options.read(true).write(access == 1);
    #[cfg(unix)]
    options.custom_flags(libc::O_NONBLOCK | libc::O_NOCTTY);
    let file = options.open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("resource must be a regular file".into());
    }
    Ok(file)
}
fn writable(turn: &Turn) -> bool {
    !turn.read_only && turn.journal.is_some()
}
pub(super) unsafe extern "C" fn open(
    ctx: *mut c_void,
    id: u64,
    bytes: Bytes,
    policy: u32,
    access: u32,
) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if !writable(turn)
        || id == 0
        || policy > 2
        || access > 1
        || turn.files.len() >= 64
        || turn.files.contains_key(&id)
        || bytes.len == 0
        || bytes.len > 4096
        || bytes.ptr.is_null()
    {
        return ERROR;
    }
    let Ok(path) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
    else {
        return ERROR;
    };
    let Ok(path) = std::fs::canonicalize(path) else {
        return ERROR;
    };
    let Ok(file) = open_regular(&path, access) else {
        return ERROR;
    };
    turn.files.insert(
        id,
        HostFile {
            file: Arc::new(file),
            path,
            policy,
            access,
        },
    );
    OK
}
pub(super) unsafe extern "C" fn close(ctx: *mut c_void, id: u64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if !writable(turn) || id == 0 {
        return ERROR;
    }
    turn.files.remove(&id);
    OK
}
pub(super) unsafe extern "C" fn read(
    ctx: *mut c_void,
    id: u64,
    offset: u64,
    out: *mut u8,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &*context.turn };
    if !writable(turn) || capacity > 1024 * 1024 || out.is_null() {
        return ERROR as isize;
    }
    let Some(resource) = turn.files.get(&id) else {
        return ERROR as isize;
    };
    let buffer = unsafe { std::slice::from_raw_parts_mut(out, capacity) };
    resource
        .file
        .read_at(buffer, offset)
        .map_or(ERROR as isize, |length| length as isize)
}
pub(super) fn replacement(
    files: &BTreeMap<u64, HostFile>,
) -> Result<BTreeMap<u64, HostFile>, String> {
    let mut next = BTreeMap::new();
    for (key, resource) in files {
        match resource.policy {
            0 => {
                next.insert(*key, resource.clone());
            }
            1 => {}
            2 => {
                next.insert(
                    *key,
                    HostFile {
                        file: Arc::new(open_regular(&resource.path, resource.access)?),
                        path: resource.path.clone(),
                        policy: 2,
                        access: resource.access,
                    },
                );
            }
            _ => return Err("invalid resource policy".into()),
        }
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::{
        ffi::CString,
        io::Read,
        os::unix::{ffi::OsStrExt, net::UnixListener},
    };

    #[test]
    fn writable_open_retains_existing_contents_and_reopen_access() {
        let root =
            std::env::temp_dir().join(format!("identity-writable-file-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("document");
        std::fs::write(&path, b"original").unwrap();
        let resource = HostFile {
            file: Arc::new(open_regular(&path, 1).unwrap()),
            path: path.clone(),
            policy: 2,
            access: 1,
        };
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        let replaced = replacement(&BTreeMap::from([(1, resource)])).unwrap();
        replaced[&1].file.write_all_at(b"changed!", 0).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"changed!");
        assert!(open_regular(&root.join("missing"), 1).is_err());
        assert!(!root.join("missing").exists());
        drop(replaced);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[cfg(unix)]
    fn opens_regular_descriptor_and_rejects_fifo_without_waiting_for_writer() {
        let root = std::env::temp_dir().join(format!("identity-file-kind-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("document");
        std::fs::write(&path, b"retained inode").unwrap();
        let mut retained = open_regular(&path, 0).unwrap();
        std::fs::remove_file(&path).unwrap();
        let name = CString::new(path.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        let started = std::time::Instant::now();
        assert!(open_regular(&path, 0).unwrap_err().contains("regular file"));
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
        let mut contents = String::new();
        retained.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "retained inode");
        assert!(open_regular(&root, 0).is_err());
        let socket = root.join("socket");
        let listener = UnixListener::bind(&socket).unwrap();
        assert!(open_regular(&socket, 0).is_err());
        drop(listener);
        std::fs::remove_dir_all(root).unwrap();
    }
}
