//! Resolution and loading of packaged JavaScript application resources.

use std::path::{Path, PathBuf};

use snafu::ResultExt;

pub(crate) fn load() -> crate::Result<String> {
    let path = path()?;
    std::fs::read_to_string(&path).context(crate::error::ReadFileSnafu {
        kind: "JavaScript bundle",
        path,
    })
}

pub(crate) fn load_source_map() -> crate::Result<Option<Vec<u8>>> {
    let path = path()?.with_extension("js.map");
    if !path.is_file() {
        return Ok(None);
    }
    std::fs::read(&path)
        .map(Some)
        .context(crate::error::ReadFileSnafu {
            kind: "JavaScript source map",
            path,
        })
}

pub(crate) fn path() -> crate::Result<PathBuf> {
    if let Some(path) = std::env::var_os("WABOU_BUNDLE_PATH") {
        return Ok(PathBuf::from(path));
    }
    let executable = std::env::current_exe().context(crate::error::ReadFileSnafu {
        kind: "current executable path",
        path: PathBuf::from("<current executable>"),
    })?;
    Ok(candidates(&executable)
        .into_iter()
        .find(|path| path.is_file())
        .unwrap_or_else(|| adjacent_path(&executable)))
}

pub(crate) fn resource_directory() -> crate::Result<PathBuf> {
    let bundle = path()?;
    Ok(bundle.parent().unwrap_or_else(|| Path::new(".")).to_owned())
}

fn adjacent_path(executable: &Path) -> PathBuf {
    executable
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("resources/bundle.js")
}

fn candidates(executable: &Path) -> Vec<PathBuf> {
    let adjacent = adjacent_path(executable);
    let Some(directory) = executable.parent() else {
        return vec![adjacent];
    };
    let mut candidates = vec![adjacent];

    // cargo-packager places Debian resources under
    // /usr/lib/<binary>/resources rather than next to /usr/bin/<binary>.
    if directory.file_name().and_then(|name| name.to_str()) == Some("bin")
        && let (Some(prefix), Some(binary)) = (directory.parent(), executable.file_stem())
    {
        candidates.push(prefix.join("lib").join(binary).join("resources/bundle.js"));
    }

    // A macOS .app keeps executables and resources in sibling directories.
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
    use super::{adjacent_path, candidates};
    use std::path::Path;

    #[test]
    fn packaged_bundle_is_resolved_next_to_the_executable() {
        assert_eq!(
            adjacent_path(Path::new("/opt/demo/demo")),
            Path::new("/opt/demo/resources/bundle.js")
        );
    }

    #[test]
    fn native_packages_expose_platform_resource_candidates() {
        assert_eq!(
            candidates(Path::new("/usr/bin/warden-desktop")),
            [
                Path::new("/usr/bin/resources/bundle.js").to_path_buf(),
                Path::new("/usr/lib/warden-desktop/resources/bundle.js").to_path_buf(),
            ]
        );
        assert_eq!(
            candidates(Path::new(
                "/Applications/Warden.app/Contents/MacOS/warden-desktop"
            )),
            [
                Path::new("/Applications/Warden.app/Contents/MacOS/resources/bundle.js")
                    .to_path_buf(),
                Path::new("/Applications/Warden.app/Contents/Resources/resources/bundle.js")
                    .to_path_buf(),
                Path::new("/Applications/Warden.app/Contents/Resources/bundle.js").to_path_buf(),
            ]
        );
    }
}
