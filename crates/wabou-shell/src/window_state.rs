//! Durable native window geometry with an intentionally small state schema.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{ExtensionContext, ShellExtension, WindowOptions, WindowResourceKey};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct SavedWindowSize {
    width: u32,
    height: u32,
}

/// Restore and persist the normal logical size of one native window.
///
/// Maximized dimensions are deliberately ignored: restoring those as the
/// normal size makes the next unmaximize operation surprising.
pub struct WindowSizePersistence {
    path: PathBuf,
    window_key: WindowResourceKey,
    last_normal_size: Option<SavedWindowSize>,
}

impl WindowSizePersistence {
    /// Restore a valid saved size into `options` and create its persistence hook.
    pub fn restore(
        path: impl Into<PathBuf>,
        window_key: WindowResourceKey,
        options: &mut WindowOptions,
    ) -> Self {
        let path = path.into();
        let saved = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SavedWindowSize>(&bytes).ok())
            .filter(valid_size);
        if let Some(saved) = saved {
            let min = options.min_inner_size.unwrap_or((1, 1));
            options.initial_inner_size = (saved.width.max(min.0), saved.height.max(min.1));
        }
        let restored = saved.map(|_| SavedWindowSize {
            width: options.initial_inner_size.0,
            height: options.initial_inner_size.1,
        });
        Self {
            path,
            window_key,
            last_normal_size: restored,
        }
    }

    fn observe(&mut self, context: &mut ExtensionContext<'_>) {
        let Some(metrics) = context.window_metrics(self.window_key) else {
            return;
        };
        if !metrics.maximized {
            let size = SavedWindowSize {
                width: metrics.logical_width,
                height: metrics.logical_height,
            };
            if valid_size(&size) {
                self.last_normal_size = Some(size);
            }
        }
    }

    fn save(&self) {
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

impl ShellExtension for WindowSizePersistence {
    fn initialize(&mut self, _wake: crate::WakeCallback) -> Result<(), String> {
        Ok(())
    }

    fn poll(&mut self, context: &mut ExtensionContext<'_>) {
        self.observe(context);
    }

    fn window_metrics_changed(
        &mut self,
        window_key: WindowResourceKey,
        context: &mut ExtensionContext<'_>,
    ) {
        if window_key == self.window_key {
            self.observe(context);
        }
    }

    fn close_requested(
        &mut self,
        window_key: WindowResourceKey,
        context: &mut ExtensionContext<'_>,
    ) -> bool {
        if window_key == self.window_key {
            self.observe(context);
            self.save();
        }
        false
    }

    fn shutdown(&mut self, context: &mut ExtensionContext<'_>) {
        self.observe(context);
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
        let mut options = WindowOptions::new().min_inner_size(900, 600);

        let _state = WindowSizePersistence::restore(
            &path,
            crate::initial_window_resource_key(0),
            &mut options,
        );

        assert_eq!(options.initial_inner_size, (900, 600));
    }

    #[test]
    fn saved_size_replaces_the_previous_file() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("state/window.json");
        let state = WindowSizePersistence {
            path: path.clone(),
            window_key: crate::initial_window_resource_key(0),
            last_normal_size: Some(SavedWindowSize {
                width: 1280,
                height: 840,
            }),
        };

        state.save();

        let saved: SavedWindowSize = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(saved.width, 1280);
        assert_eq!(saved.height, 840);
    }
}
