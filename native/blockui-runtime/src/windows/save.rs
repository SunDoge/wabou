//! Windows publication: private staging, directory pinning and ACL-preserving replacement.
use std::{
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io::Write,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use windows_sys::Win32::{
    Foundation::{GENERIC_WRITE, INVALID_HANDLE_VALUE, LocalFree},
    Security::{
        Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW, SECURITY_ATTRIBUTES,
    },
    Storage::FileSystem::{
        CREATE_NEW, CreateFileW, DELETE, FILE_ADD_FILE, FILE_ATTRIBUTE_NORMAL,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        GetFinalPathNameByHandleW,
    },
};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn wide(value: &OsStr) -> Result<Vec<u16>, String> {
    let mut value: Vec<_> = value.encode_wide().collect();
    if value.contains(&0) {
        return Err("save path contains NUL".into());
    }
    value.push(0);
    Ok(value)
}
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
struct Descriptor(*mut std::ffi::c_void);
impl Drop for Descriptor {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
fn user_sid() -> std::io::Result<String> {
    use std::os::windows::io::OwnedHandle;
    use windows_sys::Win32::{
        Security::{
            Authorization::ConvertSidToStringSidW, GetTokenInformation, TOKEN_QUERY, TOKEN_USER,
            TokenUser,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    let mut token = std::ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    let mut size = 0;
    unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            std::ptr::null_mut(),
            0,
            &mut size,
        );
    }
    if size == 0 || size > 65536 {
        return Err(std::io::Error::other("invalid process token user size"));
    }
    let mut buffer = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            buffer.as_mut_ptr().cast(),
            size,
            &mut size,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error());
    }
    let user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
    let mut text = std::ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(user.User.Sid, &mut text) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    let _allocation = Descriptor(text.cast());
    let mut length = 0;
    while unsafe { *text.add(length) } != 0 {
        length += 1;
    }
    String::from_utf16(unsafe { std::slice::from_raw_parts(text, length) })
        .map_err(std::io::Error::other)
}
fn private_file(path: &Path) -> Result<File, std::io::Error> {
    // Owner and SYSTEM only, until the completed staging file receives the destination DACL.
    let sddl = wide(OsStr::new(&format!(
        "D:P(A;;FA;;;SY)(A;;FA;;;{})",
        user_sid()?
    )))
    .unwrap();
    let mut descriptor = Descriptor(std::ptr::null_mut());
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor.0,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error());
    }
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let path = wide(path.as_os_str()).map_err(std::io::Error::other)?;
    let handle = unsafe {
        CreateFileW(
            path.as_ptr(),
            GENERIC_WRITE | DELETE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            &attributes,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_handle(handle) })
    }
}
fn copy_dacl(target: &Path, staging: &Path) -> Result<(), String> {
    use windows_sys::Win32::Security::{
        Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT, SetNamedSecurityInfoW},
        DACL_SECURITY_INFORMATION, GetSecurityDescriptorControl,
        PROTECTED_DACL_SECURITY_INFORMATION, SE_DACL_PROTECTED,
        UNPROTECTED_DACL_SECURITY_INFORMATION,
    };
    let target = wide(target.as_os_str())?;
    let staging = wide(staging.as_os_str())?;
    let mut descriptor = Descriptor(std::ptr::null_mut());
    let mut dacl = std::ptr::null_mut();
    let error = unsafe {
        GetNamedSecurityInfoW(
            target.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut dacl,
            std::ptr::null_mut(),
            &mut descriptor.0,
        )
    };
    if error != 0 {
        return Err(std::io::Error::from_raw_os_error(error as i32).to_string());
    }
    let mut control = 0;
    let mut revision = 0;
    if unsafe { GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision) } == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let inheritance = if control & SE_DACL_PROTECTED != 0 {
        PROTECTED_DACL_SECURITY_INFORMATION
    } else {
        UNPROTECTED_DACL_SECURITY_INFORMATION
    };
    let error = unsafe {
        SetNamedSecurityInfoW(
            staging.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | inheritance,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            dacl,
            std::ptr::null(),
        )
    };
    if error != 0 {
        return Err(std::io::Error::from_raw_os_error(error as i32).to_string());
    }
    Ok(())
}
fn rename_relative(
    file: &File,
    directory: &File,
    name: &OsStr,
    create: bool,
) -> Result<(), String> {
    use windows_sys::{
        Wdk::Storage::FileSystem::{
            FILE_RENAME_INFORMATION, FILE_RENAME_POSIX_SEMANTICS, FILE_RENAME_REPLACE_IF_EXISTS,
            FileRenameInformationEx, NtSetInformationFile,
        },
        Win32::{Foundation::RtlNtStatusToDosError, System::IO::IO_STATUS_BLOCK},
    };
    let name = wide(name)?;
    let name_bytes = u32::try_from((name.len() - 1) * 2).map_err(|_| "save filename too long")?;
    let size = std::mem::size_of::<FILE_RENAME_INFORMATION>() + name_bytes as usize;
    // All fields align at most to usize; reserve the variable UTF-16 tail as well.
    let mut storage = vec![0usize; size.div_ceil(std::mem::size_of::<usize>())];
    let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
    unsafe {
        (*info).Anonymous.Flags = FILE_RENAME_POSIX_SEMANTICS
            | if create {
                0
            } else {
                FILE_RENAME_REPLACE_IF_EXISTS
            };
        (*info).RootDirectory = directory.as_raw_handle();
        (*info).FileNameLength = name_bytes;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            std::ptr::addr_of_mut!((*info).FileName).cast::<u16>(),
            name.len(),
        );
    }
    let mut status: IO_STATUS_BLOCK = unsafe { std::mem::zeroed() };
    let result = unsafe {
        NtSetInformationFile(
            file.as_raw_handle(),
            &mut status,
            info.cast(),
            size as u32,
            FileRenameInformationEx,
        )
    };
    if result < 0 {
        return Err(std::io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(result) } as i32
        )
        .to_string());
    }
    Ok(())
}
fn directory_path(directory: &File) -> Result<PathBuf, String> {
    let mut path = vec![0u16; 512];
    loop {
        let size = unsafe {
            GetFinalPathNameByHandleW(
                directory.as_raw_handle(),
                path.as_mut_ptr(),
                path.len() as u32,
                0,
            )
        };
        if size == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        if (size as usize) < path.len() {
            path.truncate(size as usize);
            return Ok(PathBuf::from(OsString::from_wide(&path)));
        }
        if size > 32768 {
            return Err("save directory path too long".into());
        }
        path.resize(size as usize + 1, 0);
    }
}
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
fn publish_with(
    path: &Path,
    bytes: &[u8],
    create: bool,
    before_publish: impl FnOnce() -> std::io::Result<()>,
) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path.file_name().ok_or("save target has no filename")?;
    // Omitting SHARE_DELETE pins the directory for the complete transaction.
    let directory = OpenOptions::new()
        .read(true)
        .access_mode(windows_sys::Win32::Foundation::GENERIC_READ | FILE_ADD_FILE)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(parent)
        .map_err(|e| e.to_string())?;
    let parent = directory_path(&directory)?;
    let target = parent.join(name);
    if !create {
        let metadata = fs::symlink_metadata(&target).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("save target must be an existing regular file".into());
        }
    }
    let (mut file, temporary) = {
        let mut staged = None;
        for _ in 0..128 {
            let id = NEXT
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
                .map_err(|_| "save temporary identity exhausted")?;
            let candidate = parent.join(format!(".identity-save-{}-{id}", std::process::id()));
            match private_file(&candidate) {
                Ok(file) => {
                    staged = Some((file, Temporary(candidate)));
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.to_string()),
            }
        }
        staged.ok_or("save temporary name collisions exceeded limit")?
    };
    file.write_all(bytes).map_err(|e| e.to_string())?;
    if !create {
        copy_dacl(&target, &temporary.0).map_err(|e| format!("save DACL copy failed: {e}"))?;
    }
    file.sync_all().map_err(|e| e.to_string())?;
    before_publish().map_err(|e| e.to_string())?;
    // One handle-relative rename is atomic even with old readers, and cannot
    // follow a newly substituted parent directory or overwrite a racing create.
    rename_relative(&file, &directory, name, create)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn publication_pins_parent_and_cleans_cancelled_and_conflicting_staging() {
        let root = std::env::temp_dir().join(format!("blockui-save-basic-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let parent = root.join("草稿");
        fs::create_dir(&parent).unwrap();
        let path = parent.join("文档.txt");
        publish_with(&path, b"original", true, || {
            assert!(fs::rename(&parent, root.join("moved")).is_err());
            Ok(())
        })
        .unwrap();
        assert!(publish_cancellable(&path, b"cancelled", false, || true).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"original");
        publish_cancellable(&path, b"updated", false, || false).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"updated");
        let racing = parent.join("racing.txt");
        assert!(publish_with(&racing, b"ours", true, || fs::write(&racing, b"other")).is_err());
        assert_eq!(fs::read(&racing).unwrap(), b"other");
        assert_eq!(fs::read_dir(&parent).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn complete_publication_preserves_old_handles_and_rejects_racing_creators() {
        let root = std::env::temp_dir().join(format!("blockui-save-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let path = root.join("草稿 🦀.txt");
        publish_cancellable(&path, b"original", true, || false).unwrap();
        let original = File::open(&path).unwrap();
        assert!(publish_cancellable(&path, b"cancelled", false, || true).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        publish_cancellable(&path, b"updated", false, || false).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"updated");
        let mut old = String::new();
        std::io::Read::read_to_string(&mut &original, &mut old).unwrap();
        assert_eq!(old, "original");
        drop(original);
        let race = root.join("racing.txt");
        assert!(publish_with(&race, b"ours", true, || fs::write(&race, b"other")).is_err());
        assert_eq!(fs::read(&race).unwrap(), b"other");
        let missing = root.join("missing.txt");
        assert!(publish_cancellable(&missing, b"new", false, || false).is_err());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
