//! Filesystem publication is unavailable in browser builds.
pub(super) fn publish_cancellable(
    _: &std::path::Path,
    _: &[u8],
    _: bool,
    _: impl FnOnce() -> bool,
) -> Result<(), String> {
    Err("filesystem publication is unavailable in browser builds".into())
}
