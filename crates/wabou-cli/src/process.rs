use std::io::{BufRead, BufReader, Read};
use std::net::{TcpStream, ToSocketAddrs};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use process_wrap::std::JobObject;
#[cfg(unix)]
use process_wrap::std::ProcessGroup;
use process_wrap::std::{ChildWrapper, CommandWrap};

use super::{Result, ensure};

pub(super) fn ensure_host_exit(status: ExitStatus) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        // SIGTERM/SIGINT are cooperative shutdown requests from a terminal or
        // process supervisor. Preserve genuine crashes (including SIGKILL).
        const SIGINT: i32 = 2;
        const SIGTERM: i32 = 15;
        if matches!(status.signal(), Some(SIGTERM) | Some(SIGINT)) {
            return Ok(());
        }
    }
    ensure(status, "Rust host")
}

pub(super) struct ManagedChild {
    pub(super) child: Box<dyn ChildWrapper>,
}

impl ManagedChild {
    pub(super) fn spawn(command: Command) -> std::io::Result<Self> {
        let mut command = CommandWrap::from(command);
        #[cfg(unix)]
        command.wrap(ProcessGroup::leader());
        #[cfg(windows)]
        command.wrap(JobObject);
        command.spawn().map(|child| Self { child })
    }

    pub(super) fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        self.terminate();
    }
}

pub(super) fn configure_test_backend(command: &mut Command, native: bool) {
    if native {
        // `--native` must win over an inherited shell/CI variable.
        command.env_remove("WABOU_TEST_HEADLESS");
    } else {
        command.env("WABOU_TEST_HEADLESS", "1");
    }
}

#[cfg(test)]
pub(super) fn wait_for_managed_child(
    command: Command,
    timeout: Duration,
    stopped: &AtomicBool,
) -> Result<ExitStatus> {
    let mut child = ManagedChild::spawn(command)?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.child.try_wait()? {
            return Ok(status);
        }
        if stopped.load(Ordering::Acquire) {
            child.terminate();
            return Err("Wabou behavior test interrupted".into());
        }
        if Instant::now() >= deadline {
            child.terminate();
            return Err(format!(
                "Wabou behavior test host exceeded its final {}s watchdog",
                timeout.as_secs()
            )
            .into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn behavior_runtime_diagnostic(line: &str) -> bool {
    line.contains("[STRICT_READ_UNTRACKED]")
        || line.contains("[REACTIVITY_HALTED]")
        || line.contains("rejected runtime utility class")
}

fn forward_behavior_output(
    reader: impl Read + Send + 'static,
    stderr: bool,
    diagnostics: Arc<Mutex<Vec<String>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            let Ok(line) = line else { break };
            if behavior_runtime_diagnostic(&line)
                && let Ok(mut collected) = diagnostics.lock()
            {
                collected.push(line.clone());
            }
            if stderr {
                eprintln!("{line}");
            } else {
                println!("{line}");
            }
        }
    })
}

/// Wait for a behavior-test host while rejecting runtime diagnostics that
/// otherwise only appear as warnings beside a successful scenario report.
pub(super) fn wait_for_behavior_host(
    mut command: Command,
    timeout: Duration,
    stopped: &AtomicBool,
) -> Result<ExitStatus> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = ManagedChild::spawn(command)?;
    let diagnostics = Arc::new(Mutex::new(Vec::new()));
    let stdout = child
        .child
        .stdout()
        .take()
        .map(|output| forward_behavior_output(output, false, diagnostics.clone()));
    let stderr = child
        .child
        .stderr()
        .take()
        .map(|output| forward_behavior_output(output, true, diagnostics.clone()));
    let deadline = Instant::now() + timeout;
    let result: Result<ExitStatus> = loop {
        if let Some(status) = child.child.try_wait()? {
            break Ok(status);
        }
        if stopped.load(Ordering::Acquire) {
            child.terminate();
            break Err("Wabou behavior test interrupted".into());
        }
        if Instant::now() >= deadline {
            child.terminate();
            break Err(format!(
                "Wabou behavior test host exceeded its final {}s watchdog",
                timeout.as_secs()
            )
            .into());
        }
        thread::sleep(Duration::from_millis(10));
    };
    if let Some(stdout) = stdout {
        let _ = stdout.join();
    }
    if let Some(stderr) = stderr {
        let _ = stderr.join();
    }
    let status = result?;
    let diagnostics = diagnostics
        .lock()
        .map_err(|_| "behavior diagnostic collector lock poisoned")?;
    if !diagnostics.is_empty() {
        return Err(format!(
            "behavior host emitted runtime diagnostics:\n  - {}",
            diagnostics.join("\n  - ")
        )
        .into());
    }
    Ok(status)
}

pub(super) fn wait_for_vite(url: &str, child: &mut dyn ChildWrapper) -> Result<()> {
    let authority = url.trim_start_matches("http://");
    let address = authority
        .to_socket_addrs()?
        .next()
        .ok_or("Vite address did not resolve")?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            return Err(format!("Vite exited before startup: {status}").into());
        }
        if TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err("timed out waiting for Vite".into())
}

pub(super) fn supervise(
    host: &mut ManagedChild,
    vite: &mut ManagedChild,
    inspector: Option<&mut ManagedChild>,
) -> Result<()> {
    let stopped = Arc::new(AtomicBool::new(false));
    let signal = stopped.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;
    let mut inspector = inspector;
    let result = loop {
        if stopped.load(Ordering::Acquire) {
            break Ok(());
        }
        if let Some(status) = host.child.try_wait()? {
            break ensure_host_exit(status);
        }
        if let Some(status) = vite.child.try_wait()? {
            break ensure(status, "Vite dev server");
        }
        if let Some(child) = inspector.as_mut()
            && let Some(status) = child.child.try_wait()?
        {
            eprintln!("[wabou] DevTools exited: {status}");
            inspector = None;
        }
        thread::sleep(Duration::from_millis(50));
    };
    host.terminate();
    vite.terminate();
    if let Some(child) = inspector {
        child.terminate();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{behavior_runtime_diagnostic, ensure_host_exit};

    #[test]
    fn behavior_diagnostics_reject_reactivity_and_runtime_style_failures() {
        for line in [
            "WARN js: [STRICT_READ_UNTRACKED] direct read",
            "ERROR js: [REACTIVITY_HALTED] recursive update",
            "WARN style_resolution: rejected runtime utility class class=bad",
        ] {
            assert!(behavior_runtime_diagnostic(line), "missed {line:?}");
        }
        assert!(!behavior_runtime_diagnostic(
            "WARN rfd: zenity is not installed"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn host_sigterm_is_a_clean_supervisor_shutdown() {
        let status = std::process::Command::new("sh")
            .args(["-c", "kill -TERM $$"])
            .status()
            .unwrap();
        ensure_host_exit(status).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn host_sigkill_remains_an_error() {
        let status = std::process::Command::new("sh")
            .args(["-c", "kill -KILL $$"])
            .status()
            .unwrap();
        assert!(ensure_host_exit(status).is_err());
    }
}
