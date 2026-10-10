//! One size boundary for loader and development snapshots.
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
pub(crate) const MAX_BYTES: u64 = 32 * 1024 * 1024;

pub(crate) fn read(source: &Path) -> Result<Vec<u8>, String> {
    let file = File::open(source).map_err(|e| e.to_string())?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Err("plugin artifact must be a regular file".into());
    }
    if metadata.len() > MAX_BYTES {
        return Err("plugin exceeds 32 MiB snapshot limit".into());
    }
    // The opened file can grow after metadata is read. Bound the actual read as well.
    read_bounded(file, metadata.len())
}
fn read_bounded(reader: impl Read, expected: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity((expected.min(MAX_BYTES) + 1) as usize);
    reader
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("plugin exceeds 32 MiB snapshot limit".into());
    }
    Ok(bytes)
}
pub(crate) fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err("plugin exceeds 32 MiB snapshot limit".into());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    if let Err(error) = file.write_all(bytes) {
        #[cfg(not(target_arch = "wasm32"))]
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(error.to_string());
    }
    Ok(())
}
#[cfg(feature = "native-window")]
pub(crate) fn copy(source: &Path, destination: &Path) -> Result<(), String> {
    let bytes = read(source)?;
    write(destination, &bytes)
}

/// Allocated storage rather than the logical file length.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn allocated(path: &Path) -> Result<u64, String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(path)
            .map(|m| m.blocks() * 512)
            .map_err(|e| e.to_string())
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_STANDARD_INFO, FileStandardInfo, GetFileInformationByHandleEx,
        };
        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut info: FILE_STANDARD_INFO = unsafe { std::mem::zeroed() };
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileStandardInfo,
                (&mut info as *mut FILE_STANDARD_INFO).cast(),
                std::mem::size_of_val(&info) as u32,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
        u64::try_from(info.AllocationSize).map_err(|_| "negative file allocation size".into())
    }
}

#[cfg(windows)]
pub(crate) fn mapped(directory: &Path) -> Result<usize, String> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW,
                TH32CS_SNAPMODULE,
            },
            Threading::GetCurrentProcessId,
        },
    };
    struct Snapshot(windows_sys::Win32::Foundation::HANDLE);
    impl Drop for Snapshot {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, GetCurrentProcessId()) };
    if handle == INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let snapshot = Snapshot(handle);
    let marker = format!(
        "\\{}\\impl-",
        directory
            .file_name()
            .ok_or("missing actor directory")?
            .to_string_lossy()
    )
    .to_lowercase();
    let mut entry: MODULEENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of_val(&entry) as u32;
    let mut found = 0;
    let mut next = unsafe { Module32FirstW(snapshot.0, &mut entry) };
    if next == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    while next != 0 {
        let end = entry
            .szExePath
            .iter()
            .position(|c| *c == 0)
            .unwrap_or(entry.szExePath.len());
        if String::from_utf16_lossy(&entry.szExePath[..end])
            .to_lowercase()
            .contains(&marker)
        {
            found += 1;
        }
        next = unsafe { Module32NextW(snapshot.0, &mut entry) };
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_growth_after_metadata_without_reading_unbounded_data() {
        struct Growing {
            consumed: u64,
        }
        impl Read for Growing {
            fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
                self.consumed += out.len() as u64;
                out.fill(0);
                Ok(out.len())
            }
        }
        let mut growing = Growing { consumed: 0 };
        assert!(
            read_bounded(&mut growing, MAX_BYTES)
                .unwrap_err()
                .contains("32 MiB")
        );
        assert_eq!(growing.consumed, MAX_BYTES + 1);
    }
    #[test]
    fn rejects_oversize_and_preserves_existing_snapshot() {
        let dir =
            std::env::temp_dir().join(format!("identity-artifact-test-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let source = dir.join("oversize.so");
        File::create(&source)
            .unwrap()
            .set_len(MAX_BYTES + 1)
            .unwrap();
        assert!(read(&source).unwrap_err().contains("32 MiB"));
        let destination = dir.join("snapshot.so");
        write(&destination, b"old immutable bytes").unwrap();
        assert!(write(&destination, b"replacement").is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), b"old immutable bytes");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
