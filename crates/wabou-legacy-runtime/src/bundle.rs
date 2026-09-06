//! Resolution of the JavaScript bundle used by the Winit application host.

use std::path::{Path, PathBuf};

use snafu::ResultExt;

pub(crate) fn load() -> crate::Result<String> {
    let path = path()?;
    std::fs::read_to_string(&path).context(crate::error::ReadFileSnafu {
        kind: "JavaScript bundle",
        path,
    })
}

fn path() -> crate::Result<PathBuf> {
    if let Some(path) = std::env::var_os("WABOU_BUNDLE_PATH") {
        return Ok(PathBuf::from(path));
    }
    let executable = std::env::current_exe().context(crate::error::ReadFileSnafu {
        kind: "current executable path",
        path: PathBuf::from("<current executable>"),
    })?;
    Ok(bundle_candidates(&executable)
        .into_iter()
        .find(|path| path.is_file())
        .unwrap_or_else(|| adjacent_bundle(&executable)))
}

fn adjacent_bundle(executable: &Path) -> PathBuf {
    executable
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("resources/bundle.js")
}

fn bundle_candidates(executable: &Path) -> Vec<PathBuf> {
    let adjacent = adjacent_bundle(executable);
    let Some(directory) = executable.parent() else {
        return vec![adjacent];
    };
    let mut candidates = vec![adjacent];
    if directory.file_name().and_then(|name| name.to_str()) == Some("bin")
        && let (Some(prefix), Some(binary)) = (directory.parent(), executable.file_stem())
    {
        candidates.push(prefix.join("lib").join(binary).join("resources/bundle.js"));
    }
    if directory.file_name().and_then(|name| name.to_str()) == Some("MacOS")
        && let Some(contents) = directory.parent()
    {
        candidates.push(contents.join("Resources/resources/bundle.js"));
        candidates.push(contents.join("Resources/bundle.js"));
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_bundle_next_to_a_development_binary() {
        assert_eq!(
            adjacent_bundle(Path::new("/tmp/demo/demo")),
            Path::new("/tmp/demo/resources/bundle.js")
        );
    }
}
