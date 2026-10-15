//! Portable frontend-only distribution using the installed CLI as its host.
use super::*;

pub(super) fn bundle(workspace: &Path, app: &App, configuration: &Path, out: &Path) -> Result<()> {
    #[cfg(not(feature = "embedded-renderer"))]
    let _ = (workspace, app, configuration, out);
    #[cfg(not(feature = "embedded-renderer"))]
    return Err("bundle requires a CLI with embedded-renderer".into());

    #[cfg(feature = "embedded-renderer")]
    {
        if out.exists() {
            return Err(format!("output already exists: {}", out.display()).into());
        }
        let mut config = load_launch_config(configuration)?;
        ensure(
            build_frontend(workspace, app, &[], BuildProfile::Release, false)?,
            "Vite build",
        )?;

        let parent = out
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let staging = tempfile::tempdir_in(parent)?;
        let ui_bundle = bundle_path(workspace, app, BuildProfile::Release)?;
        config.source = wabou_launch::UiSource::Bundle {
            path: PathBuf::from("resources")
                .join(ui_bundle.file_name().ok_or("bundle has no filename")?),
        };
        copy_tree(
            ui_bundle.parent().ok_or("bundle has no directory")?,
            &staging.path().join("resources"),
        )?;
        config.resource_directory = Some("resources".into());
        fs::write(
            staging.path().join("app.json"),
            serde_json::to_vec_pretty(&config)?,
        )?;

        let executable = if cfg!(windows) { "wabou.exe" } else { "wabou" };
        fs::copy(env::current_exe()?, staging.path().join(executable))?;
        fs::write(
            staging.path().join("README.txt"),
            format!("Run: {executable} launch app.json\nResource files belong in resources/.\n"),
        )?;
        fs::rename(staging.path(), out)?;
        println!("[wabou] portable frontend preview: {}", out.display());
        Ok(())
    }
}

#[cfg(feature = "embedded-renderer")]
fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
