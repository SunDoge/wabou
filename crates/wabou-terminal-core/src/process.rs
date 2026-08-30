//! Terminal launch configuration and PTY child-process lifecycle.

use std::path::PathBuf;
#[cfg(unix)]
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LaunchConfig {
    pub(super) command: String,
    pub(super) args: Vec<String>,
    pub(super) cwd: Option<String>,
    pub(super) login_shell: bool,
}

impl LaunchConfig {
    pub(super) fn default_shell() -> Self {
        Self {
            command: default_shell_command(),
            args: Vec::new(),
            cwd: None,
            login_shell: true,
        }
    }
}

pub(super) fn pty_spawn_parts(launch: &LaunchConfig) -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        (
            quote_windows_command_arg(&launch.command),
            launch
                .args
                .iter()
                .map(|arg| quote_windows_command_arg(arg))
                .collect(),
        )
    }
    #[cfg(not(windows))]
    {
        let mut args = vec![
            "TERM=xterm-256color".into(),
            "COLORTERM=truecolor".into(),
            "TERM_PROGRAM=wabou".into(),
            format!("TERM_PROGRAM_VERSION={}", env!("CARGO_PKG_VERSION")),
            launch.command.clone(),
        ];
        #[cfg(target_os = "macos")]
        if launch.login_shell {
            args.push("-l".into());
        }
        args.extend(launch.args.iter().cloned());
        ("/usr/bin/env".into(), args)
    }
}

pub(super) fn validate_working_directory(cwd: Option<&str>) -> std::io::Result<()> {
    let Some(cwd) = cwd else {
        return Ok(());
    };
    let path = std::path::Path::new(cwd);
    let metadata = std::fs::metadata(path).map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!(
                "terminal working directory `{}` is unavailable: {error}",
                path.display()
            ),
        )
    })?;
    if !metadata.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            format!(
                "terminal working directory `{}` is not a directory",
                path.display()
            ),
        ));
    }
    Ok(())
}

/// Quote one argv item for the Windows `CommandLineToArgvW` rules.
#[cfg(any(windows, test))]
pub(super) fn quote_windows_command_arg(value: &str) -> String {
    if !value.is_empty()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte == b'"')
    {
        return value.to_owned();
    }
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    let mut backslashes = 0;
    for character in value.chars() {
        if character == '\\' {
            backslashes += 1;
            continue;
        }
        if character == '"' {
            quoted.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
        } else {
            quoted.extend(std::iter::repeat_n('\\', backslashes));
        }
        backslashes = 0;
        quoted.push(character);
    }
    quoted.extend(std::iter::repeat_n('\\', backslashes * 2));
    quoted.push('"');
    quoted
}

#[cfg(unix)]
pub(super) fn validate_launch_command(command: &str) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let path = PathBuf::from(command);
    let candidates: Vec<PathBuf> = if path.components().count() > 1 {
        vec![path]
    } else {
        std::env::var_os("PATH")
            .map(|path| {
                std::env::split_paths(&path)
                    .map(|directory| directory.join(command))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut found_non_executable = false;
    for candidate in candidates {
        let Ok(metadata) = candidate.metadata() else {
            continue;
        };
        if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
            return Ok(());
        }
        found_non_executable = true;
    }
    let kind = if found_non_executable {
        std::io::ErrorKind::PermissionDenied
    } else {
        std::io::ErrorKind::NotFound
    };
    Err(std::io::Error::new(
        kind,
        format!("terminal command is not executable: {command}"),
    ))
}

#[cfg(not(unix))]
pub(super) fn validate_launch_command(_command: &str) -> std::io::Result<()> {
    Ok(())
}

#[cfg(unix)]
pub(super) fn spawn_child_reaper(
    pid: libc::pid_t,
) -> std::io::Result<std::thread::JoinHandle<std::io::Result<()>>> {
    const SHUTDOWN_GRACE: Duration = Duration::from_millis(500);
    std::thread::Builder::new()
        .name("PTY child reaper".into())
        .spawn(move || {
            let deadline = Instant::now() + SHUTDOWN_GRACE;
            let mut sent_hangup = false;
            loop {
                let mut status = 0;
                let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
                if result == pid {
                    return Ok(());
                }
                if result == 0 {
                    if !sent_hangup {
                        // create_pty_with_spawn calls setsid in the child, so
                        // its PID is also the process-group id.
                        unsafe { libc::kill(-pid, libc::SIGHUP) };
                        sent_hangup = true;
                    }
                    if Instant::now() >= deadline {
                        unsafe {
                            libc::kill(-pid, libc::SIGKILL);
                            libc::kill(pid, libc::SIGKILL);
                        }
                        loop {
                            let result = unsafe { libc::waitpid(pid, &mut status, 0) };
                            if result == pid {
                                return Ok(());
                            }
                            let error = std::io::Error::last_os_error();
                            if error.kind() == std::io::ErrorKind::Interrupted {
                                continue;
                            }
                            if error.raw_os_error() == Some(libc::ECHILD) {
                                return Ok(());
                            }
                            return Err(error);
                        }
                    }
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                if error.raw_os_error() == Some(libc::ECHILD) {
                    return Ok(());
                }
                return Err(error);
            }
        })
}

pub(super) fn default_shell_command() -> String {
    #[cfg(windows)]
    {
        std::env::var("COMSPEC")
            .ok()
            .filter(|command| !command.is_empty())
            .unwrap_or_else(|| "cmd.exe".into())
    }
    #[cfg(not(windows))]
    {
        std::env::var("SHELL")
            .ok()
            .filter(|command| !command.is_empty())
            .unwrap_or_else(|| "/bin/sh".into())
    }
}
