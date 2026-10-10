//! Positional file operations using explicit Windows offsets, never seek+read pairs.
use std::{fs::File, io, os::windows::fs::FileExt as WindowsFileExt};
pub(crate) trait FileExt {
    fn read_at(&self, bytes: &mut [u8], offset: u64) -> io::Result<usize>;
    fn write_all_at(&self, bytes: &[u8], offset: u64) -> io::Result<()>;
}
impl FileExt for File {
    fn read_at(&self, bytes: &mut [u8], offset: u64) -> io::Result<usize> {
        self.seek_read(bytes, offset)
    }
    fn write_all_at(&self, mut bytes: &[u8], mut offset: u64) -> io::Result<()> {
        while !bytes.is_empty() {
            match self.seek_write(bytes, offset) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "file wrote zero bytes",
                    ));
                }
                Ok(length) => {
                    offset = offset
                        .checked_add(length as u64)
                        .ok_or_else(|| io::Error::other("file offset overflow"))?;
                    bytes = &bytes[length..];
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}
