//! Own process groups and bounded captured logs for development builds/metadata.
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
    time::Duration,
};
#[cfg(windows)]
#[path = "windows/process.rs"]
mod windows;
const STDOUT_LIMIT: u64 = 2 * 1024 * 1024;
const STDERR_LIMIT: u64 = 4 * 1024 * 1024;
static NEXT: AtomicU64 = AtomicU64::new(0);
#[cfg(unix)]
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}
struct Logs(PathBuf, PathBuf);
impl Logs {
    fn exceeds_limit(&self) -> bool {
        fs::metadata(&self.0).is_ok_and(|file| file.len() > STDOUT_LIMIT)
            || fs::metadata(&self.1).is_ok_and(|file| file.len() > STDERR_LIMIT)
    }
}
struct ProcessGroup(Child, bool, #[cfg(windows)] windows::Job);
impl ProcessGroup {
    fn spawn(command: &mut Command) -> std::io::Result<Self> {
        #[cfg(unix)]
        {
            Ok(Self(command.spawn()?, false))
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let job = windows::Job::new()?;
            command.creation_flags(windows_sys::Win32::System::Threading::CREATE_SUSPENDED);
            let process = Self(command.spawn()?, false, job);
            process.2.assign_and_resume(&process.0)?;
            Ok(process)
        }
    }
    fn terminate(&mut self) -> std::io::Result<std::process::ExitStatus> {
        #[cfg(unix)]
        unsafe {
            kill(-(self.0.id() as i32), 9);
        }
        #[cfg(windows)]
        {
            self.2.terminate();
            let _ = self.0.kill();
        }
        let result = self.0.wait();
        self.1 = result.is_ok();
        result
    }
    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        #[cfg(target_os = "linux")]
        {
            // Observe exit without releasing the leader PID. Its process-group
            // identity remains reserved until descendants have been signaled.
            let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
            let result = unsafe {
                libc::waitid(
                    libc::P_PID,
                    self.0.id() as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            };
            if result < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if unsafe { info.si_pid() } == 0 {
                return Ok(None);
            }
            self.terminate().map(Some)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let status = self.0.try_wait()?;
            self.1 = status.is_some();
            Ok(status)
        }
    }
}
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        if !self.1 {
            let _ = self.terminate();
        }
    }
}
impl Drop for Logs {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_file(&self.1);
    }
}
pub(crate) fn output(
    command: &mut Command,
    stop: &AtomicBool,
    directory: &Path,
) -> Result<Output, String> {
    output_until(command, stop, directory, || false)
}

/// Cancel only this command's owned process group when its input becomes obsolete.
pub(crate) fn output_until(
    command: &mut Command,
    stop: &AtomicBool,
    directory: &Path,
    mut obsolete: impl FnMut() -> bool,
) -> Result<Output, String> {
    if stop.load(Ordering::Relaxed) {
        return Err("development command canceled".into());
    }
    if obsolete() {
        return Err("development build superseded".into());
    }
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let prefix = format!(
        "watch-log-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let logs = Logs(
        directory.join(format!("{prefix}.out")),
        directory.join(format!("{prefix}.err")),
    );
    let stdout = File::create(&logs.0).map_err(|error| error.to_string())?;
    let stderr = File::create(&logs.1).map_err(|error| error.to_string())?;
    command
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .stdin(Stdio::null());
    #[cfg(unix)]
    command.process_group(0);
    let mut process = ProcessGroup::spawn(command).map_err(|error| error.to_string())?;
    let status = loop {
        let superseded = obsolete();
        if stop.load(Ordering::Relaxed) || superseded || logs.exceeds_limit() {
            // Only the freshly spawned command's own process group, including cargo/rustc children.
            let _ = process.terminate();
            return Err(if stop.load(Ordering::Relaxed) {
                "development command canceled"
            } else if superseded {
                "development build superseded"
            } else {
                "development command exceeded log limit"
            }
            .into());
        }
        match process.try_wait().map_err(|error| error.to_string())? {
            Some(status) => break status,
            // Poll only while an owned development command is pending.
            // Shorter completion detection avoids a 20 ms quantization delay.
            None => thread::sleep(Duration::from_millis(5)),
        }
    };
    // The command can write its final output between the poll above and exit.
    // Reject overflow before exposing potentially truncated metadata as success.
    if logs.exceeds_limit() {
        return Err("development command exceeded log limit".into());
    }
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    File::open(&logs.0)
        .map_err(|error| error.to_string())?
        .take(STDOUT_LIMIT)
        .read_to_end(&mut stdout)
        .map_err(|error| error.to_string())?;
    File::open(&logs.1)
        .map_err(|error| error.to_string())?
        .take(STDERR_LIMIT)
        .read_to_end(&mut stderr)
        .map_err(|error| error.to_string())?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

/// Keep failure context even when Cargo exits without writing stderr.
pub(crate) fn failure(operation: &str, output: &Output) -> String {
    let mut message = format!("Cargo {operation} failed ({})", output.status);
    let details = if output.stderr.is_empty() {
        &output.stdout
    } else {
        &output.stderr
    };
    if !details.is_empty() {
        message.push('\n');
        message.push_str(&String::from_utf8_lossy(details));
    }
    diagnostic(message.as_bytes())
}

/// Keep both the first diagnostic and the final failure after noisy warnings.
/// The display remains byte-bounded and truncation never splits a UTF-8 scalar.
pub(crate) fn diagnostic(stderr: &[u8]) -> String {
    const LIMIT: usize = 8000;
    const OMITTED: &str = "\n… diagnostics truncated; showing first and last output …\n";
    let text = String::from_utf8_lossy(stderr);
    if text.len() <= LIMIT {
        return text.into_owned();
    }
    let available = LIMIT - OMITTED.len();
    let mut first = available / 2;
    while !text.is_char_boundary(first) {
        first -= 1;
    }
    let mut last = text.len() - (available - first);
    while !text.is_char_boundary(last) {
        last += 1;
    }
    let mut result = String::with_capacity(LIMIT);
    result.push_str(&text[..first]);
    result.push_str(OMITTED);
    result.push_str(&text[last..]);
    result
}

#[cfg(test)]
mod diagnostic_tests {
    use super::*;
    #[test]
    fn short_diagnostics_preserve_exact_output() {
        let message = "error: 类型不匹配 🦀\n --> src/lib.rs:12:5\n";
        assert_eq!(diagnostic(message.as_bytes()), message);
        assert_eq!(diagnostic(b""), "");
    }
    #[test]
    fn noisy_diagnostics_preserve_the_final_error_and_unicode_within_budget() {
        let message = format!(
            "{}error: final type mismatch\n --> src/lib.rs:12:5\n",
            "warning: 中文 🦀\n".repeat(2000)
        );
        let excerpt = diagnostic(message.as_bytes());
        assert!(excerpt.starts_with("warning: 中文 🦀"));
        assert!(excerpt.ends_with("error: final type mismatch\n --> src/lib.rs:12:5\n"));
        assert!(excerpt.contains("diagnostics truncated"));
        assert!(excerpt.len() <= 8000);
        assert!(!excerpt.contains('\u{fffd}'));
    }
    #[test]
    fn invalid_utf8_output_stays_lossy_and_bounded() {
        let mut message = vec![0xff; 9000];
        message.extend_from_slice(b"\nerror: failed\n");
        let excerpt = diagnostic(&message);
        assert!(excerpt.len() <= 8000);
        assert!(excerpt.ends_with("error: failed\n"));
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn completed_output_respects_both_log_limits() {
        let directory = std::env::temp_dir().join(format!(
            "identity-completed-log-proof-{}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        for (stream, limit) in [("out", STDOUT_LIMIT), ("err", STDERR_LIMIT)] {
            for extra in [0, 1] {
                let mut command = Command::new("python3");
                command.args(["-c", "import sys; stream=sys.stderr.buffer if sys.argv[1]=='err' else sys.stdout.buffer; stream.write(b'x'*int(sys.argv[2])); stream.flush()"])
                    .arg(stream).arg((limit + extra).to_string());
                let result = output(&mut command, &AtomicBool::new(false), &directory);
                if extra == 0 {
                    let result = result.unwrap();
                    assert!(result.status.success());
                    assert_eq!(
                        result.stdout.len(),
                        if stream == "out" { limit as usize } else { 0 }
                    );
                } else {
                    assert_eq!(
                        result.unwrap_err(),
                        "development command exceeded log limit"
                    );
                }
                assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
            }
        }
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn silent_failed_command_keeps_operation_and_exit_status() {
        let directory =
            std::env::temp_dir().join(format!("identity-silent-failure-{}", std::process::id()));
        let mut command = Command::new("python3");
        command.args(["-c", "import sys; sys.exit(19)"]);
        let result = output(&mut command, &AtomicBool::new(false), &directory).unwrap();
        assert!(result.stderr.is_empty());
        let message = failure("build", &result);
        assert!(message.contains("Cargo build failed"), "{message}");
        assert!(message.contains("19"), "{message}");
        let mut command = Command::new("python3");
        command.args([
            "-c",
            "import sys; print('wrapper: 编译器不可用'); sys.exit(19)",
        ]);
        let result = output(&mut command, &AtomicBool::new(false), &directory).unwrap();
        assert!(result.stderr.is_empty());
        let message = failure("build", &result);
        assert!(message.contains("wrapper: 编译器不可用"), "{message}");
        assert!(message.contains("19"), "{message}");
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn noisy_failed_command_keeps_the_error_after_the_old_capture_cutoff() {
        let directory =
            std::env::temp_dir().join(format!("identity-noisy-diagnostic-{}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let mut command = Command::new("python3");
        command.args(["-c", "import sys; sys.stderr.write('warning: noisy dependency\\n'*2000); sys.stderr.write('error: final application failure\\n'); sys.exit(1)"]);
        let result = output(&mut command, &AtomicBool::new(false), &directory).unwrap();
        assert!(!result.status.success());
        assert!(result.stderr.len() > 8000);
        let message = failure("build", &result);
        assert!(message.starts_with("Cargo build failed"));
        assert!(message.contains("warning: noisy dependency"));
        assert!(message.ends_with("error: final application failure\n"));
        assert!(message.len() <= 8000);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
        fs::remove_dir(directory).unwrap();
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn completed_command_terminates_background_descendants_and_removes_logs() {
        let directory = std::env::temp_dir().join(format!(
            "identity-exited-process-proof-{}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        let mut command = Command::new("python3");
        command.args(["-c", "import subprocess,sys; p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); print(p.pid,flush=True)"]);
        let result = output(&mut command, &AtomicBool::new(false), &directory).unwrap();
        assert!(result.status.success());
        let pid: i32 = String::from_utf8(result.stdout)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let running = || {
            fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|stat| {
                !matches!(
                    stat.split_once(") ").unwrap().1.chars().next(),
                    Some('Z' | 'X')
                )
            })
        };
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while running() && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        let running = running();
        if running {
            unsafe {
                kill(pid, 9);
            }
        }
        let logs = fs::read_dir(&directory).unwrap().count();
        fs::remove_dir_all(directory).unwrap();
        assert!(!running, "finished build left a running descendant");
        assert_eq!(logs, 0);
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn log_overflow_terminates_descendants_cleans_storage_and_allows_next_command() {
        for stderr in [false, true] {
            let directory = std::env::temp_dir().join(format!(
                "identity-log-limit-proof-{}-{stderr}",
                std::process::id()
            ));
            fs::create_dir(&directory).unwrap();
            let ready = directory.join("child.pid");
            let mut command = Command::new("python3");
            command.args(["-c", "import subprocess,sys,pathlib,time; p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); pathlib.Path(sys.argv[1]).write_text(str(p.pid)); stream=sys.stderr.buffer if sys.argv[2]=='err' else sys.stdout.buffer; stream.write(b'x'*(int(sys.argv[3])+1)); stream.flush(); time.sleep(60)"])
                .arg(&ready).arg(if stderr { "err" } else { "out" })
                .arg(if stderr { "4194304" } else { "2097152" });
            let error = output(&mut command, &AtomicBool::new(false), &directory).unwrap_err();
            let pid: i32 = fs::read_to_string(&ready).unwrap().parse().unwrap();
            let running = || {
                fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|stat| {
                    !matches!(
                        stat.split_once(") ").unwrap().1.chars().next(),
                        Some('Z' | 'X')
                    )
                })
            };
            let deadline = std::time::Instant::now() + Duration::from_secs(1);
            while running() && std::time::Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            let remained = running();
            if remained {
                unsafe {
                    kill(pid, 9);
                }
            }
            assert_eq!(error, "development command exceeded log limit");
            assert!(!remained);
            assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
            let mut next = Command::new("python3");
            next.args(["-c", "print('healthy')"]);
            let result = output(&mut next, &AtomicBool::new(false), &directory).unwrap();
            assert!(result.status.success());
            assert_eq!(result.stdout, b"healthy\n");
            assert!(result.stderr.is_empty());
            assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
            fs::remove_dir_all(directory).unwrap();
        }
    }
    #[test]
    fn cancellation_terminates_owned_process_group_and_removes_logs() {
        process_group_proof(false);
    }
    #[test]
    fn supersession_terminates_owned_process_group_and_removes_logs() {
        process_group_proof(true);
    }
    fn process_group_proof(supersession: bool) {
        let directory = std::env::temp_dir().join(format!(
            "identity-process-proof-{}-{supersession}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        let ready = directory.join("ready");
        let stopped = std::sync::Arc::new(AtomicBool::new(false));
        let stop = stopped.clone();
        let root = directory.clone();
        let signal = ready.clone();
        let worker = thread::spawn(move || {
            let mut command = Command::new("python3");
            command.args(["-c", "import pathlib,subprocess,sys,time,os; p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); pathlib.Path(sys.argv[1]).write_text(f'{os.getpid()} {p.pid}'); time.sleep(60)"]).arg(signal);
            if supersession {
                output_until(&mut command, &AtomicBool::new(false), &root, || {
                    stop.load(Ordering::Relaxed)
                })
            } else {
                output(&mut command, &stop, &root)
            }
        });
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !ready.exists() && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        let pids: Vec<u32> = fs::read_to_string(&ready)
            .unwrap()
            .split_whitespace()
            .map(|pid| pid.parse().unwrap())
            .collect();
        assert_eq!(pids.len(), 2);
        for pid in &pids {
            assert!(Path::new(&format!("/proc/{pid}")).exists());
        }
        let started = std::time::Instant::now();
        stopped.store(true, Ordering::Relaxed);
        let error = worker.join().unwrap().unwrap_err();
        assert!(error.contains(if supersession {
            "superseded"
        } else {
            "canceled"
        }));
        assert!(started.elapsed() < Duration::from_secs(2));
        for pid in pids {
            let running = || {
                fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|stat| {
                    !matches!(
                        stat.split_once(") ").unwrap().1.chars().next(),
                        Some('Z' | 'X')
                    )
                })
            };
            // SIGKILL is asynchronous for descendants: waiting for the group
            // leader does not guarantee the child has received CPU to exit.
            let deadline = std::time::Instant::now() + Duration::from_secs(1);
            while running() && std::time::Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            assert!(!running(), "canceled build left a running descendant {pid}");
        }
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1); // only our readiness file
        fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use windows_sys::Win32::{
        Foundation::WAIT_OBJECT_0,
        System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
    };
    const HELPER: &str = "dev_process::windows_tests::owned_process_helper";
    #[test]
    #[ignore = "child process fixture, invoked by the job ownership test"]
    fn owned_process_helper() {
        let marker = std::env::var_os("BLOCKUI_JOB_TEST_MARKER").unwrap();
        if std::env::var_os("BLOCKUI_JOB_TEST_CHILD").is_none() {
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", HELPER, "--ignored"])
                .env("BLOCKUI_JOB_TEST_CHILD", "1")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            fs::write(marker, child.id().to_string()).unwrap();
            let _ = child.wait();
        }
        thread::sleep(Duration::from_secs(60));
    }
    #[test]
    fn supersession_terminates_the_owned_windows_job_and_descendants() {
        let directory = std::env::temp_dir().join(format!("blockui-job-{}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let marker = directory.join("child.pid");
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", HELPER, "--ignored"])
            .env("BLOCKUI_JOB_TEST_MARKER", &marker);
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let mut descendant = None;
        let error = output_until(&mut command, &AtomicBool::new(false), &directory, || {
            if let Ok(pid) = fs::read_to_string(&marker) {
                let pid = pid.parse().unwrap();
                let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
                if !handle.is_null() {
                    descendant = Some(unsafe { OwnedHandle::from_raw_handle(handle) });
                }
                return true;
            }
            std::time::Instant::now() >= deadline
        })
        .unwrap_err();
        assert_eq!(error, "development build superseded");
        let descendant = descendant.expect("build did not launch its descendant");
        assert_eq!(
            unsafe { WaitForSingleObject(descendant.as_raw_handle(), 2000) },
            WAIT_OBJECT_0
        );
        assert_eq!(
            fs::read_dir(&directory).unwrap().count(),
            1,
            "build logs were not removed"
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
