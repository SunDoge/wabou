use std::{fs::File, io};
pub(crate) trait FileExt {
    fn read_at(&self, _: &mut [u8], _: u64) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "browser filesystem is unavailable",
        ))
    }
    fn write_all_at(&self, _: &[u8], _: u64) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "browser filesystem is unavailable",
        ))
    }
}
impl FileExt for File {}
