//! Backend-independent persistence for a window's normal logical size.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct SavedWindowSize {
    width: u32,
    height: u32,
}

/// Restores and records the normal logical size of one native window.
///
/// The GPUI shell feeds authoritative resize/maximize observations into this
/// object. Persistence remains independent from application state so restoring
/// a window never depends on booting the JavaScript runtime first.
pub struct WindowSizePersistence {
    path: PathBuf,
    last_normal_size: Option<SavedWindowSize>,
}

impl WindowSizePersistence {
    /// Load a valid saved size and clamp it to the current minimum.
    #[must_use]
    pub fn restore(
        path: impl Into<PathBuf>,
        initial_size: (u32, u32),
        minimum_size: Option<(u32, u32)>,
    ) -> (Self, (u32, u32)) {
        let path = path.into();
        let saved = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SavedWindowSize>(&bytes).ok())
            .filter(valid_size);
        let minimum = minimum_size.unwrap_or((1, 1));
        let restored = saved.map_or(initial_size, |saved| {
            (saved.width.max(minimum.0), saved.height.max(minimum.1))
        });
        let last_normal_size = saved.map(|_| SavedWindowSize {
            width: restored.0,
            height: restored.1,
        });
        (
            Self {
                path,
                last_normal_size,
            },
            restored,
        )
    }

    /// Record an authoritative logical viewport size when it is a normal window.
    pub fn observe(&mut self, width: u32, height: u32, maximized: bool) {
        let size = SavedWindowSize { width, height };
        if !maximized && valid_size(&size) {
            self.last_normal_size = Some(size);
        }
    }

    /// Atomically persist the last observed normal size, if any.
    pub fn save(&self) {
        let Some(size) = self.last_normal_size else {
            return;
        };
        if let Some(parent) = self.path.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            tracing::warn!(path = %parent.display(), %error, "failed to create window state directory");
            return;
        }
        let temporary = self.path.with_extension("json.tmp");
        let result = serde_json::to_vec(&size)
            .map_err(std::io::Error::other)
            .and_then(|bytes| std::fs::write(&temporary, bytes))
            .and_then(|()| replace_file(&temporary, &self.path));
        if let Err(error) = result {
            tracing::warn!(path = %self.path.display(), %error, "failed to persist window size");
        }
    }
}

impl Drop for WindowSizePersistence {
    fn drop(&mut self) {
        self.save();
    }
}

fn valid_size(size: &SavedWindowSize) -> bool {
    (1..=16_384).contains(&size.width) && (1..=16_384).contains(&size.height)
}

fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    if cfg!(windows) && destination.exists() {
        std::fs::remove_file(destination)?;
    }
    std::fs::rename(source, destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_clamps_to_the_current_minimum_size() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("window.json");
        std::fs::write(&path, br#"{"width":640,"height":480}"#).unwrap();

        let (_state, restored) =
            WindowSizePersistence::restore(&path, (1280, 840), Some((900, 600)));

        assert_eq!(restored, (900, 600));
    }

    #[test]
    fn maximized_observations_do_not_replace_the_normal_size() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("state/window.json");
        let (mut state, _) = WindowSizePersistence::restore(&path, (1280, 840), None);
        state.observe(1280, 840, false);
        state.observe(1920, 1080, true);
        state.save();

        let saved: SavedWindowSize = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(saved.width, 1280);
        assert_eq!(saved.height, 840);
    }
}
